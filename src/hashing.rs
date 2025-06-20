use std::fs::File;
use std::io::{self, Read, BufReader};
use sha2::{Sha256, Digest as ShaDigest};
use blake3;
use xxhash_rust::xxh3::Xxh3;
use rayon::prelude::*;

#[derive(Clone, Debug)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
    XxHash3,
}

pub fn hash_file(path: &str, algo: HashAlgorithm) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    match algo {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            let mut buffer = [0u8; 8192];
            loop {
                let n = reader.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        },
        HashAlgorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            let mut buffer = [0u8; 8192];
            loop {
                let n = reader.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().as_bytes().to_vec())
        },
        HashAlgorithm::XxHash3 => {
            let mut hasher = Xxh3::new();
            let mut buffer = [0u8; 8192];
            loop {
                let n = reader.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.digest().to_le_bytes().to_vec())
        },
    }
}

pub fn hash_file_sha256(path: &str) -> std::io::Result<Vec<u8>> {
    use sha2::{Sha256, Digest};
    use std::fs::File;
    use std::io::{BufReader, Read};
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.finalize().to_vec())
}

pub fn hash_file_blake3(path: &str) -> std::io::Result<Vec<u8>> {
    use blake3;
    use std::fs::File;
    use std::io::{BufReader, Read};
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.finalize().as_bytes().to_vec())
}

pub fn hash_file_xxhash3(path: &str) -> std::io::Result<Vec<u8>> {
    use xxhash_rust::xxh3::Xxh3;
    use std::fs::File;
    use std::io::{BufReader, Read};
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Xxh3::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.digest().to_le_bytes().to_vec())
}

pub fn hash_files_parallel(paths: &[&str], algo: HashAlgorithm) -> Vec<(String, Vec<u8>)> {
    paths.par_iter()
        .map(|path| {
            let hash = hash_file(path, algo.clone()).unwrap_or_default();
            (path.to_string(), hash)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use hex;
    use tempfile::NamedTempFile;    

    #[test]
    fn test_hash_file_sha256() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "hello world").unwrap();
        let hash = hash_file_sha256(file.path().to_str().unwrap()).unwrap();
        // SHA-256 of "hello world"
        let expected = vec![
            0xb9, 0x4d, 0x27, 0xb9, 0x93, 0x4d, 0x3e, 0x08,
            0xa5, 0x2e, 0x52, 0xd7, 0xda, 0x7d, 0xab, 0xfa,
            0xc4, 0x84, 0xef, 0xe3, 0x7a, 0x53, 0x80, 0xee,
            0x90, 0x88, 0xf7, 0xac, 0xe2, 0xef, 0xcd, 0xe9
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hash_file_blake3() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "hello world").unwrap();
        let hash = hash_file_blake3(file.path().to_str().unwrap()).unwrap();
        let expected = hex::decode("d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24").unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hash_file_xxhash3() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "hello world").unwrap();
        let hash = hash_file_xxhash3(file.path().to_str().unwrap()).unwrap();
        // xxHash3 64-bit of "hello world"
        let expected = vec![0x8b, 0x98, 0xe6, 0x40, 0xea, 0xb1, 0x47, 0xd4];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hash_file_enum_dispatch() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "hello world").unwrap();
        let path = file.path().to_str().unwrap();

        let sha256 = hash_file(path, HashAlgorithm::Sha256).unwrap();
        let blake3 = hash_file(path, HashAlgorithm::Blake3).unwrap();
        let xxhash3 = hash_file(path, HashAlgorithm::XxHash3).unwrap();

        let expected_sha256 = vec![
            0xb9, 0x4d, 0x27, 0xb9, 0x93, 0x4d, 0x3e, 0x08,
            0xa5, 0x2e, 0x52, 0xd7, 0xda, 0x7d, 0xab, 0xfa,
            0xc4, 0x84, 0xef, 0xe3, 0x7a, 0x53, 0x80, 0xee,
            0x90, 0x88, 0xf7, 0xac, 0xe2, 0xef, 0xcd, 0xe9
        ];
        let expected_blake3 = hex::decode("d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24").unwrap();
        let expected_xxhash3 = vec![0x8b, 0x98, 0xe6, 0x40, 0xea, 0xb1, 0x47, 0xd4];

        assert_eq!(sha256, expected_sha256);
        assert_eq!(blake3, expected_blake3);
        assert_eq!(xxhash3, expected_xxhash3);
    }

    #[test]
    fn test_hash_files_parallel() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        write!(file1, "hello world").unwrap();
        write!(file2, "rustacean").unwrap();
        let paths = [file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let results = hash_files_parallel(&paths, HashAlgorithm::Sha256);
        let expected1 = vec![
            0xb9, 0x4d, 0x27, 0xb9, 0x93, 0x4d, 0x3e, 0x08,
            0xa5, 0x2e, 0x52, 0xd7, 0xda, 0x7d, 0xab, 0xfa,
            0xc4, 0x84, 0xef, 0xe3, 0x7a, 0x53, 0x80, 0xee,
            0x90, 0x88, 0xf7, 0xac, 0xe2, 0xef, 0xcd, 0xe9
        ];
        let expected2 = vec![
            0x6e, 0x7e, 0x0e, 0x6e, 0x7e, 0x0e, 0x6e, 0x7e,
            0x0e, 0x6e, 0x7e, 0x0e, 0x6e, 0x7e, 0x0e, 0x6e,
            0x7e, 0x0e, 0x6e, 0x7e, 0x0e, 0x6e, 0x7e, 0x0e,
            0x6e, 0x7e, 0x0e, 0x6e, 0x7e, 0x0e, 0x6e, 0x7e
        ]; // Placeholder, will update below
        assert_eq!(results[0].1, expected1);
        // Compute expected2 using hash_file_sha256 for "rustacean"
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "rustacean").unwrap();
        let expected2 = hash_file_sha256(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(results[1].1, expected2);
    }
}