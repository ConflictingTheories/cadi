//! Hashing utilities for CADI

use sha2::{Sha256, Digest};
use std::io::{Read, BufReader};
use std::path::Path;

/// Compute SHA256 hash of bytes
pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Compute SHA256 hash of a string
pub fn sha256_str(s: &str) -> String {
    sha256_bytes(s.as_bytes())
}

/// Compute SHA256 hash of a file
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

/// Create a chunk ID from a hash
pub fn chunk_id_from_hash(hash: &str) -> String {
    format!("chunk:sha256:{}", hash)
}

/// Create a chunk ID from content
pub fn chunk_id_from_content(content: &[u8]) -> String {
    chunk_id_from_hash(&sha256_bytes(content))
}

/// Parse a chunk ID to extract the hash
pub fn parse_chunk_id(chunk_id: &str) -> Option<String> {
    chunk_id
        .strip_prefix("chunk:sha256:")
        .map(|s| s.to_string())
}

/// Verify that content matches a chunk ID
pub fn verify_chunk_content(chunk_id: &str, content: &[u8]) -> bool {
    if let Some(expected_hash) = parse_chunk_id(chunk_id) {
        let actual_hash = sha256_bytes(content);
        expected_hash == actual_hash
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_bytes() {
        let hash = sha256_bytes(b"hello world");
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_chunk_id() {
        let hash = sha256_bytes(b"test");
        let chunk_id = chunk_id_from_hash(&hash);
        assert!(chunk_id.starts_with("chunk:sha256:"));
        assert_eq!(parse_chunk_id(&chunk_id), Some(hash));
    }

    #[test]
    fn test_verify_content() {
        let content = b"test content";
        let chunk_id = chunk_id_from_content(content);
        assert!(verify_chunk_content(&chunk_id, content));
        assert!(!verify_chunk_content(&chunk_id, b"different content"));
    }
}
