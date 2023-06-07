use std::io::{Read, Write};

extern crate fatfs;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// disk file
    #[arg(short, long)]
    disk: String,

    /// binary files.
    #[arg(short, long)]
    file: String,

    /// filename in the disk
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let img_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&cli.disk)
        .unwrap();
    let fs = fatfs::FileSystem::new(img_file, fatfs::FsOptions::new()).unwrap();

    let root_dir = fs.root_dir();

    let output = cli
        .output
        .unwrap_or(cli.file.rsplitn(2, '/').next().unwrap().into());

    let mut read_file = std::fs::OpenOptions::new()
        .read(true)
        .open(&cli.file)
        .unwrap();
    let mut buf: Vec<u8> = Vec::new();

    read_file.read_to_end(&mut buf).unwrap();

    let mut write_file = root_dir.create_file(&output).unwrap();
    write_file.write_all(&buf).unwrap();
}
