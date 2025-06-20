use std::fs::File;
use std::io::{self, Read, BufReader};
use sha2::{Sha256, Digest as ShaDigest};
use blake3;
use xxhash_rust::xxh3::Xxh3;

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
        let expected = hex::decode("e167f68d2b2c1f0debafaf1b1b6c890bd1e3a3a2b6b8e176b6c6d2a3a0a1ec09").unwrap();
        assert_eq!(hash, expected);
    }
}