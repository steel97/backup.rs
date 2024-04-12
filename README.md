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

server requirements:
```
+ debian based OS
+ pg_dump/mysqldump for databases
```