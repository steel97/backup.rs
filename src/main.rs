use anyhow::{anyhow, Context, Result};
use aws_sdk_s3::{
    error::SdkError,
    operation::put_object::{PutObjectError, PutObjectOutput},
    primitives::ByteStream,
    Client,
};
use backup::{core::constants::VERSION, packaging::archive::Archive};
use chrono::{Datelike, Timelike};
use rand::{distributions::Alphanumeric, Rng};
use std::{
    collections::HashMap,
    env::{self, set_var},
    fs,
    io::{self, Write},
    path::Path,
    process::Command,
};

#[::tokio::main]
async fn main() -> Result<()> {
    println!("backup.rs v{}", VERSION);
    // load backup configuration
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow!("Error! No config specified."));
    }

    if !std::path::Path::new(&args[1]).exists() {
        println!("Error! Config {} doesn't exists", &args[1]);
        return Err(anyhow!("Error! No config specified."));
    }

    let model = backup::models::config::Config::read_config(&args[1])
        .with_context(|| format!("Failed to read instrs from {}", &args[1]))?;
    let targets = model.targets.with_context(|| format!("Missing targets"))?;
    let storages = model
        .storages
        .with_context(|| format!("Missing storages"))?;

    for target in &targets {
        let target_key = target.name.clone().unwrap_or(String::from("backup"));

        let current_date = chrono::Utc::now();
        let year = current_date.year();
        let month = format!("{:02}", current_date.month());
        let day = format!("{:02}", current_date.day());
        let hour = format!("{:02}", current_date.hour());
        let minute = format!("{:02}", current_date.minute());
        let second = format!("{:02}", current_date.second());

        let target_key = target_key
            .replace("{year}", year.to_string().as_str())
            .replace("{month}", month.as_str())
            .replace("{day}", day.as_str())
            .replace("{hour}", hour.as_str())
            .replace("{minute}", minute.as_str())
            .replace("{second}", second.as_str());

        println!("Creating {}", target_key);

        let backup_actions = match &target.backup {
            Some(res) => res,
            None => {
                println!("No backup actions for {}", target_key);
                continue;
            }
        };

        // prepare future archive
        let dir = env::temp_dir();
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let path_arch = dir.join(s.as_str());
        let pathstr = path_arch.to_str();
        let archive_path = pathstr.unwrap_or_default();
        //println!("temp dir: {}", pathstr.unwrap_or_default());
        let password = match &target.packer {
            Some(val) => match &val.password {
                Some(pwd) => pwd.clone(),
                None => String::from(""),
            },
            None => String::from(""),
        };
        let mut archive = Archive::new(String::from(archive_path), password)?;

        // pack directories
        match &backup_actions.directories {
            Some(res) => {
                for action_dir in res {
                    println!(
                        "Adding directory {}",
                        action_dir.source.clone().unwrap_or_default()
                    );
                    let source = action_dir.source.clone().unwrap_or_default();
                    let output = action_dir.output.clone().unwrap_or_default();
                    archive.pack_dir(source, output, String::from(""))?;
                }
            }
            None => {
                println!("No dirs for {}", target_key);
            }
        };

        // execute commands
        match &backup_actions.commands {
            Some(res) => {
                for command in res {
                    println!(
                        "Executing command {}",
                        command.command.clone().unwrap_or_default()
                    );

                    let cmd = command.command.clone().unwrap_or_default();
                    let output = command.output.clone().unwrap_or_default();

                    let dir = env::temp_dir();
                    let s: String = rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(16)
                        .map(char::from)
                        .collect();
                    let path_tmp = dir.join(s.as_str());

                    let mut args: Vec<String> = Vec::new();
                    for arg in command.args.clone().unwrap_or_default() {
                        args.push(arg.replace("%BKP_CMD_TMPFILE%", path_tmp.to_str().unwrap()));
                    }

                    let mut envs: HashMap<String, String> = env::vars().collect();
                    for env in command.env.clone().unwrap_or_default() {
                        envs.insert(
                            env.0,
                            env.1
                                .replace("%BKP_CMD_TMPFILE%", path_tmp.to_str().unwrap()),
                        );
                    }

                    let out = Command::new(cmd).args(args).envs(envs).output()?;
                    println!("status: {}", out.status);
                    io::stdout().write_all(&out.stdout).unwrap();
                    io::stderr().write_all(&out.stderr).unwrap();

                    if path_tmp.exists() {
                        archive.pack_file(String::from(path_tmp.to_str().unwrap()), output)?;
                        fs::remove_file(path_tmp)?;
                    } else {
                        println!("Output file not exists, skipping {}", output);
                    }
                }
            }
            None => {
                println!("No dirs for {}", target_key);
            }
        };
        // end archive
        archive.finish()?;
        // upload to storages
        for storage in &storages {
            // initialize storage (based on type TO-DO, currently S3 only)
            let access_key = <Option<std::string::String> as Clone>::clone(&storage.access_key)
                .with_context(|| format!("Missing access_key"))?;
            let secret_access_key =
                <Option<std::string::String> as Clone>::clone(&storage.secret_key)
                    .with_context(|| format!("Missing secret_key"))?;
            let region = <Option<std::string::String> as Clone>::clone(&storage.region)
                .with_context(|| format!("Missing region"))?;
            let endpoint = <Option<std::string::String> as Clone>::clone(&storage.endpoint)
                .with_context(|| format!("Missing endpoint"))?;
            set_var("AWS_ACCESS_KEY_ID", access_key);
            set_var("AWS_SECRET_ACCESS_KEY", secret_access_key);
            set_var("AWS_REGION", region);
            set_var("AWS_ENDPOINT_URL", endpoint);

            // upload built archive
            let composite_key =
                storage.key_prefix.clone().unwrap_or(String::from("")) + target_key.as_str();
            let config = aws_config::load_from_env().await;
            let client = aws_sdk_s3::Client::new(&config);

            let bucket = storage
                .bucket
                .clone()
                .with_context(|| format!("No bucket specified!"))?;

            println!("Uploading {}", composite_key);

            'upload_object: loop {
                let res = match upload_object(
                    &client,
                    bucket.as_str(),
                    archive_path,
                    composite_key.as_str(),
                )
                .await
                {
                    Ok(res) => res,
                    Err(err) => {
                        println!("Upload failed: {}", err.to_string());
                        continue 'upload_object;
                    }
                };

                println!("Uploaded {}", res.checksum_sha256.unwrap_or_default());
                break 'upload_object;
            }
        }

        if path_arch.exists() {
            fs::remove_file(path_arch)?;
        }
    }

    Ok(())
}

pub async fn upload_object(
    client: &Client,
    bucket_name: &str,
    file_name: &str,
    key: &str,
) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
    let body = ByteStream::from_path(Path::new(file_name)).await;
    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body.unwrap())
        .send()
        .await
}
