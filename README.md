# backup.rs
[![Build](https://github.com/steel97/backup.rs/actions/workflows/build.yaml/badge.svg)](https://github.com/steel97/backup.rs/actions/workflows/build.yaml)

simple backup tool (assumed to be used with crontab and S3 compatible storage)

usage:
```
cargo run <config_file>
```
Example configs: [example.json](example.json), [example_withcomments.json](example_withcomments.json)

build:
```
cargo build --release
```

requirements:
```
+ pg_dump/mysqldump for databases (optional)
```

download latest release (linux):
```
wget https://github.com/steel97/backup.rs/releases/latest/download/backup-linux-x86_64-glibc.zip -O backuprs-tool.zip && unzip backuprs-tool.zip && chmod +x ./backup && rm backuprs-tool.zip
```