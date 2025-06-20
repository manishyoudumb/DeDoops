use std::path::PathBuf;
use clap::Parser;
use crate::hashing::{hash_files_parallel, HashAlgorithm};
use std::env;
use hex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]

pub struct App {
    #[arg(short, long)]
    pub filetypes: Option<String>,

    #[arg(short, long)]
    pub dry: bool,

    #[arg(short, long)]
    pub dir: Option<PathBuf>,

}

pub fn run() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <sha256|blake3|xxhash3> <file1> [file2 ...]", args[0]);
        return;
    }
    let algo = match args[1].as_str() {
        "sha256" => HashAlgorithm::Sha256,
        "blake3" => HashAlgorithm::Blake3,
        "xxhash3" => HashAlgorithm::XxHash3,
        _ => {
            eprintln!("Unknown algorithm: {}", args[1]);
            return;
        }
    };
    let files: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
    let results = hash_files_parallel(&files, algo);
    for (path, hash) in results {
        println!("{} {}", hex::encode(hash), path);
    }
}