use std::path::PathBuf;
use clap::Parser;
use crate::hashing::{hash_files_parallel, HashAlgorithm};
use std::env;
use hex;
use std::fs;
use std::path::Path;
use walkdir;
use indicatif::{ProgressBar, ProgressStyle};

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

fn collect_files_recursively<P: AsRef<Path>>(root: P) -> Vec<String> {
    let mut files = Vec::new();
    let walker = walkdir::WalkDir::new(root).into_iter();
    for entry in walker {
        if let Ok(e) = entry {
            if e.file_type().is_file() {
                files.push(e.path().to_string_lossy().to_string());
            }
        }
    }
    files
}

pub fn run() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <sha256|blake3|xxhash3> <file|dir|drive> [file2 ...]", args[0]);
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
    let mut files: Vec<String> = Vec::new();
    let scan_target;
    if args.len() == 3 && Path::new(&args[2]).is_dir() {
        scan_target = format!("directory: {}", args[2]);
        files = collect_files_recursively(&args[2]);
    } else if args.len() == 3 && Path::new(&args[2]).is_file() {
        scan_target = format!("file: {}", args[2]);
        files.push(args[2].clone());
    } else {
        scan_target = format!("files: {}", args[2..].join(", "));
        for f in &args[2..] {
            if Path::new(f).is_file() {
                files.push(f.clone());
            }
        }
    }
    if files.is_empty() {
        eprintln!("No files found to hash.");
        return;
    }
    println!("\n=== DEDOOPs File Hasher ===");
    println!("Algorithm: {:?}", algo);
    println!("Scanning {}", scan_target);
    println!("Files to process: {}\n", files.len());
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("##-"));
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let mut results = Vec::with_capacity(files.len());
    for f in &file_refs {
        let hash = crate::hashing::hash_file(f, algo.clone()).unwrap_or_default();
        results.push((f.to_string(), hash));
        pb.inc(1);
    }
    pb.finish_with_message("done");
    for (path, hash) in &results {
        println!("{} {}", hex::encode(hash), path);
    }
    println!("\nProcessed {} files.", results.len());
}