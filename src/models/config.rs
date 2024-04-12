// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::Config;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: Config = serde_json::from_str(&json).unwrap();
// }

use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    pub storages: Option<Vec<Storage>>,
    pub targets: Option<Vec<Target>>,
}

impl Config {
    pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader)?;
        Ok(u)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Storage {
    #[serde(rename = "Type")]
    pub storage_type: Option<String>,
    pub endpoint: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub region: Option<String>,
    pub bucket: Option<String>,
    pub key_prefix: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Target {
    pub name: Option<String>,
    pub packer: Option<Packer>,
    pub backup: Option<Backup>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Backup {
    pub directories: Option<Vec<Directory>>,
    pub commands: Option<Vec<Command>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Command {
    pub output: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Directory {
    pub output: Option<String>,
    pub source: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Packer {
    pub password: Option<String>,
}
