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

fn collect_files_recursively_with_filter<P: AsRef<Path>>(root: P, exts: Option<&Vec<String>>) -> Vec<String> {
    let mut files = Vec::new();
    let debug = std::env::var("DEDOOPS_DEBUG").ok().as_deref() == Some("1");
    let walker = walkdir::WalkDir::new(root).into_iter();
    for entry in walker {
        if let Ok(e) = entry {
            if e.file_type().is_file() {
                if let Some(exts) = exts {
                    if let Some(ext) = e.path().extension().and_then(|s| s.to_str()) {
                        if debug {
                            println!("[DEBUG] Checking file: {:?}, ext: {:?}", e.path(), ext);
                        }
                        if exts.iter().any(|x| x.eq_ignore_ascii_case(ext)) {
                            files.push(e.path().to_string_lossy().to_string());
                        }
                    } else if debug {
                        println!("[DEBUG] Skipping file (no ext): {:?}", e.path());
                    }
                } else {
                    if debug {
                        println!("[DEBUG] Including file: {:?}", e.path());
                    }
                    files.push(e.path().to_string_lossy().to_string());
                }
            }
        }
    }
    files
}

pub fn run() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <sha256|blake3|xxhash3> <file|dir|drive> [file2 ...] [--filetypes=ext1,ext2]", args[0]);
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
    // Parse filetypes from --filetypes=ext1,ext2 if present
    let mut filetypes: Option<Vec<String>> = None;
    for arg in &args {
        if let Some(rest) = arg.strip_prefix("--filetypes=") {
            filetypes = Some(rest.split(',').map(|s| s.trim().to_string()).collect());
        }
    }
    let mut files: Vec<String> = Vec::new();
    let scan_target;
    if Path::new(&args[2]).is_dir() {
        scan_target = format!("directory: {}", args[2]);
        files = collect_files_recursively_with_filter(&args[2], filetypes.as_ref());
    } else if Path::new(&args[2]).is_file() {
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
    if let Some(ref exts) = filetypes {
        println!("Filtering by file types: {:?}", exts);
    }
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
    println!("\nProcessed {} files.", results.len());
}