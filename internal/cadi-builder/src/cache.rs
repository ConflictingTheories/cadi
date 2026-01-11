//! Build cache for CADI

use cadi_core::{CadiError, CadiResult, sha256_bytes};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

/// Build cache for storing and retrieving built artifacts
pub struct BuildCache {
    cache_dir: PathBuf,
}

impl BuildCache {
    /// Create a new build cache
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Check if a chunk exists in cache
    pub fn has(&self, chunk_id: &str) -> CadiResult<bool> {
        let path = self.chunk_path(chunk_id);
        Ok(path.exists())
    }

    /// Retrieve a chunk from cache
    pub fn get(&self, chunk_id: &str) -> CadiResult<Option<Vec<u8>>> {
        let path = self.chunk_path(chunk_id);
        if !path.exists() {
            return Ok(None);
        }

        let mut file = fs::File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        
        // Verify hash matches
        let expected_hash = chunk_id.strip_prefix("chunk:sha256:")
            .ok_or_else(|| CadiError::InvalidChunkId(chunk_id.to_string()))?;
        let actual_hash = sha256_bytes(&data);
        
        if expected_hash != actual_hash {
            tracing::warn!("Cache corruption detected for {}", chunk_id);
            fs::remove_file(&path)?;
            return Ok(None);
        }
        
        Ok(Some(data))
    }

    /// Store a chunk in cache
    pub fn store(&self, chunk_id: &str, data: &[u8]) -> CadiResult<()> {
        let path = self.chunk_path(chunk_id);
        
        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = fs::File::create(&path)?;
        file.write_all(data)?;
        
        Ok(())
    }

    /// Remove a chunk from cache
    pub fn remove(&self, chunk_id: &str) -> CadiResult<bool> {
        let path = self.chunk_path(chunk_id);
        if path.exists() {
            fs::remove_file(&path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CadiResult<super::CacheStats> {
        let mut total_entries = 0;
        let mut total_size_bytes = 0u64;
        
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    for subentry in fs::read_dir(&path)? {
                        let subentry = subentry?;
                        if subentry.path().is_file() {
                            total_entries += 1;
                            total_size_bytes += subentry.metadata()?.len();
                        }
                    }
                }
            }
        }
        
        Ok(super::CacheStats {
            total_entries,
            total_size_bytes,
            hit_rate: 0.0, // Would need tracking to compute
        })
    }

    /// Run garbage collection on the cache
    pub fn gc(&self, aggressive: bool) -> CadiResult<GcResult> {
        let mut removed = 0;
        let mut freed_bytes = 0u64;
        
        if !self.cache_dir.exists() {
            return Ok(GcResult {
                removed,
                freed_bytes,
            });
        }
        
        // Simple GC: remove old entries
        // In aggressive mode, remove more entries
        let max_age = if aggressive {
            std::time::Duration::from_secs(60 * 60 * 24) // 1 day
        } else {
            std::time::Duration::from_secs(60 * 60 * 24 * 7) // 7 days
        };
        
        let now = std::time::SystemTime::now();
        
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                for subentry in fs::read_dir(&path)? {
                    let subentry = subentry?;
                    let subpath = subentry.path();
                    
                    if subpath.is_file() {
                        if let Ok(metadata) = subentry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(age) = now.duration_since(modified) {
                                    if age > max_age {
                                        let size = metadata.len();
                                        if fs::remove_file(&subpath).is_ok() {
                                            removed += 1;
                                            freed_bytes += size;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(GcResult {
            removed,
            freed_bytes,
        })
    }

    /// Clear all cached chunks
    pub fn clear(&self) -> CadiResult<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Get the path for a chunk
    fn chunk_path(&self, chunk_id: &str) -> PathBuf {
        // Use first 2 chars of hash as subdirectory for better filesystem performance
        let hash = chunk_id.strip_prefix("chunk:sha256:")
            .unwrap_or(chunk_id);
        
        let prefix = if hash.len() >= 2 {
            &hash[..2]
        } else {
            "00"
        };
        
        self.cache_dir.join("chunks").join(prefix).join(hash)
    }
}

/// Result of garbage collection
#[derive(Debug)]
pub struct GcResult {
    pub removed: usize,
    pub freed_bytes: u64,
}
