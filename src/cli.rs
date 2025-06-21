use std::path::PathBuf;
use clap::Parser;
use crate::hashing::{hash_files_parallel, HashAlgorithm};
use std::env;
use hex;
use std::fs;
use std::path::Path;
use walkdir;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{SystemTime, UNIX_EPOCH};
use regex::Regex;
use serde::Serialize;

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

#[derive(Serialize)]
struct FileHashReport {
    file: String,
    hash: String,
}

fn parse_size_arg(arg: &str) -> Option<u64> {
    arg.parse::<u64>().ok()
}

fn parse_age_arg(arg: &str) -> Option<u64> {
    arg.parse::<u64>().ok()
}

fn collect_files_recursively_with_filter<P: AsRef<Path>>(root: P, exts: Option<&Vec<String>>, min_size: Option<u64>, max_size: Option<u64>, min_age: Option<u64>, max_age: Option<u64>, regex_filter: Option<&Regex>) -> Vec<String> {
    let mut files = Vec::new();
    let debug = std::env::var("DEDOOPS_DEBUG").ok().as_deref() == Some("1");
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let walker = walkdir::WalkDir::new(root).into_iter();
    for entry in walker {
        if let Ok(e) = entry {
            if e.file_type().is_file() {
                let meta = e.metadata().ok();
                let size_ok = meta.as_ref().map(|m| {
                    let len = m.len();
                    (min_size.map_or(true, |min| len >= min)) && (max_size.map_or(true, |max| len <= max))
                }).unwrap_or(false);
                if !size_ok {
                    if debug {
                        println!("[DEBUG] Skipping file (size): {:?}", e.path());
                    }
                    continue;
                }
                let age_ok = meta.as_ref().map(|m| {
                    if let Ok(modified) = m.modified() {
                        if let Ok(modified_secs) = modified.duration_since(UNIX_EPOCH) {
                            let age_secs = if now >= modified_secs.as_secs() {
                                now - modified_secs.as_secs()
                            } else {
                                0 // treat future files as age 0 days
                            };
                            let age_days = age_secs / 86400;
                            (min_age.map_or(true, |min| age_days >= min)) && (max_age.map_or(true, |max| age_days <= max))
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                }).unwrap_or(false);
                if !age_ok {
                    if debug {
                        println!("[DEBUG] Skipping file (age): {:?}", e.path());
                    }
                    continue;
                }
                let path_str = e.path().to_string_lossy();
                if let Some(re) = regex_filter {
                    if !re.is_match(&path_str) {
                        if debug {
                            println!("[DEBUG] Skipping file (regex): {:?}", e.path());
                        }
                        continue;
                    }
                }
                if let Some(exts) = exts {
                    if let Some(ext) = e.path().extension().and_then(|s| s.to_str()) {
                        if debug {
                            println!("[DEBUG] Checking file: {:?}, ext: {:?}", e.path(), ext);
                        }
                        if exts.iter().any(|x| x.eq_ignore_ascii_case(ext)) {
                            files.push(path_str.to_string());
                        }
                    } else if debug {
                        println!("[DEBUG] Skipping file (no ext): {:?}", e.path());
                    }
                } else {
                    if debug {
                        println!("[DEBUG] Including file: {:?}", e.path());
                    }
                    files.push(path_str.to_string());
                }
            }
        }
    }
    files
}

pub fn run() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <sha256|blake3|xxhash3> <file|dir|drive> [file2 ...] [--filetypes=ext1,ext2] [--min-size=BYTES] [--max-size=BYTES] [--min-age=DAYS] [--max-age=DAYS] [--regex=PATTERN] [--dry] [--quarantine-dir=PATH] [--json-report=PATH]", args[0]);
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
    let mut filetypes: Option<Vec<String>> = None;
    let mut min_size: Option<u64> = None;
    let mut max_size: Option<u64> = None;
    let mut min_age: Option<u64> = None;
    let mut max_age: Option<u64> = None;
    let mut regex_filter: Option<Regex> = None;
    let mut dry_run = false;
    let mut quarantine_dir: Option<String> = None;
    let mut json_report: Option<String> = None;
    for arg in &args {
        if let Some(rest) = arg.strip_prefix("--filetypes=") {
            filetypes = Some(rest.split(',').map(|s| s.trim().to_string()).collect());
        }
        if let Some(rest) = arg.strip_prefix("--min-size=") {
            min_size = parse_size_arg(rest);
        }
        if let Some(rest) = arg.strip_prefix("--max-size=") {
            max_size = parse_size_arg(rest);
        }
        if let Some(rest) = arg.strip_prefix("--min-age=") {
            min_age = parse_age_arg(rest);
        }
        if let Some(rest) = arg.strip_prefix("--max-age=") {
            max_age = parse_age_arg(rest);
        }
        if let Some(rest) = arg.strip_prefix("--regex=") {
            if let Ok(re) = Regex::new(rest) {
                regex_filter = Some(re);
            } else {
                eprintln!("Invalid regex pattern: {}", rest);
                return;
            }
        }
        if arg == "--dry" {
            dry_run = true;
        }
        if let Some(rest) = arg.strip_prefix("--quarantine-dir=") {
            quarantine_dir = Some(rest.to_string());
        }
        if let Some(rest) = arg.strip_prefix("--json-report=") {
            json_report = Some(rest.to_string());
        }
    }
    let mut files: Vec<String> = Vec::new();
    let scan_target;
    if Path::new(&args[2]).is_dir() {
        scan_target = format!("directory: {}", args[2]);
        files = collect_files_recursively_with_filter(&args[2], filetypes.as_ref(), min_size, max_size, min_age, max_age, regex_filter.as_ref());
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
    if let Some(min) = min_size {
        println!("Filtering by min size: {} bytes", min);
    }
    if let Some(max) = max_size {
        println!("Filtering by max size: {} bytes", max);
    }
    if let Some(min) = min_age {
        println!("Filtering by min age: {} days", min);
    }
    if let Some(max) = max_age {
        println!("Filtering by max age: {} days", max);
    }
    if let Some(ref re) = regex_filter {
        println!("Filtering by regex: {}", re);
    }
    if let Some(ref qdir) = quarantine_dir {
        println!("Quarantine directory: {}", qdir);
    }
    if let Some(ref jpath) = json_report {
        println!("JSON report: {}", jpath);
    }
    println!("Files to process: {}\n", files.len());
    if dry_run {
        println!("[DRY RUN] The following files would be processed:");
        for f in &files {
            println!("{}", f);
        }
        println!("\n[DRY RUN] {} files would be processed.", files.len());
        return;
    }
    if let Some(ref qdir) = quarantine_dir {
        fs::create_dir_all(qdir).ok();
        let mut moved = 0;
        for f in &files {
            if let Some(fname) = Path::new(f).file_name() {
                let dest = Path::new(qdir).join(fname);
                if let Err(e) = fs::rename(f, &dest) {
                    eprintln!("Failed to move {} to {}: {}", f, dest.display(), e);
                } else {
                    println!("{} -> {}", f, dest.display());
                    moved += 1;
                }
            }
        }
        println!("\n{} files moved to quarantine.", moved);
        return;
    }
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("##-"));
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let mut results = Vec::with_capacity(files.len());
    let mut report = Vec::with_capacity(files.len());
    for f in &file_refs {
        let hash = crate::hashing::hash_file(f, algo.clone()).unwrap_or_default();
        results.push((f.to_string(), hash.clone()));
        if json_report.is_some() {
            report.push(FileHashReport {
                file: f.to_string(),
                hash: hex::encode(hash),
            });
        }
        pb.inc(1);
    }
    pb.finish_with_message("done");
    if let Some(ref jpath) = json_report {
        if let Ok(json) = serde_json::to_string_pretty(&report) {
            if let Err(e) = fs::write(jpath, json) {
                eprintln!("Failed to write JSON report: {}", e);
            } else {
                println!("JSON report written to {}", jpath);
            }
        } else {
            eprintln!("Failed to serialize JSON report");
        }
    }
    println!("\nProcessed {} files.", results.len());
}

