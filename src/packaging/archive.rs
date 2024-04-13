use anyhow::Result;
use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
};
use zip::{result::ZipResult, unstable::write::FileOptionsExt, write::FileOptions, ZipWriter};

pub struct Archive {
    writer: ZipWriter<File>,
    options: FileOptions,
    buffer: Vec<u8>,
}

impl Archive {
    pub fn new<'a>(archive_path: String, password: String) -> Result<Archive> {
        let file = File::create(archive_path)?;
        let zip = ZipWriter::new(file);
        let options = if password.is_empty() {
            FileOptions::default().compression_method(zip::CompressionMethod::Bzip2)
        } else {
            FileOptions::default()
                .compression_method(zip::CompressionMethod::Bzip2)
                .with_deprecated_encryption(password.as_bytes())
        };
        let buffer = Vec::new();
        Ok(Archive {
            writer: zip,
            options,
            buffer,
        })
    }

    pub fn pack_file(&mut self, target_file_path: String, archive_file_path: String) -> Result<()> {
        let file = File::open(target_file_path)?;
        let mut reader = BufReader::new(file);
        self.writer.start_file(archive_file_path, self.options)?;
        reader.read_to_end(&mut self.buffer)?;
        self.writer.write_all(&self.buffer)?;
        self.buffer.clear();
        Ok(())
    }

    pub fn pack_dir(
        &mut self,
        target_dir_path: String,
        archive_subdir: String,
        recursive_subdir: String,
    ) -> Result<()> {
        let mut sub_dir = archive_subdir.clone();
        if !sub_dir.is_empty() && !sub_dir.ends_with("/") {
            sub_dir = sub_dir + "/";
        }

        let mut rec_subdir = recursive_subdir.clone();
        if !rec_subdir.is_empty() && !rec_subdir.ends_with("/") {
            rec_subdir = rec_subdir + "/";
        }

        let read_path = target_dir_path.clone() + rec_subdir.as_str();
        //println!("{}", read_path);
        for entry in fs::read_dir(read_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let path_clone = path.clone();
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let file = File::open(path_clone)?;
                let mut reader = BufReader::new(file);
                let archive_path_base = if rec_subdir.starts_with("/") {
                    String::from(&sub_dir[0..sub_dir.len() - 1])
                } else {
                    sub_dir.clone()
                };
                let archive_path = archive_path_base + rec_subdir.as_str() + file_name;
                self.writer.start_file(archive_path, self.options)?;
                reader.read_to_end(&mut self.buffer)?;
                self.writer.write_all(&self.buffer)?;
                self.buffer.clear();
            } else if path.is_dir() {
                let dir_name = path.file_name().unwrap().to_str().unwrap();
                let dir_path = rec_subdir.clone() + "/" + dir_name;
                //println!("{}", dir_path);
                self.pack_dir(target_dir_path.clone(), archive_subdir.clone(), dir_path)?;
            }
        }

        Ok(())
    }

    pub fn finish(&mut self) -> ZipResult<File> {
        self.writer.finish()
    }
}
