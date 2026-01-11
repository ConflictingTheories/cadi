//! Build engine for CADI

use cadi_core::{CadiError, CadiResult, Manifest};
use std::path::PathBuf;

/// Build engine configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Maximum parallel jobs
    pub parallel_jobs: usize,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Whether to use remote cache
    pub use_remote_cache: bool,
    /// Whether to fail fast on first error
    pub fail_fast: bool,
    /// Verbosity level
    pub verbose: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            parallel_jobs: num_cpus::get(),
            cache_dir: dirs::cache_dir()
                .map(|d| d.join("cadi"))
                .unwrap_or_else(|| PathBuf::from(".cadi-cache")),
            use_remote_cache: true,
            fail_fast: false,
            verbose: false,
        }
    }
}

/// Build result
#[derive(Debug)]
pub struct BuildResult {
    /// Successfully built chunks
    pub built: Vec<String>,
    /// Chunks retrieved from cache
    pub cached: Vec<String>,
    /// Failed builds
    pub failed: Vec<BuildFailure>,
    /// Total build time in milliseconds
    pub duration_ms: u64,
}

/// A build failure
#[derive(Debug)]
pub struct BuildFailure {
    pub chunk_id: String,
    pub error: String,
}

/// Build engine
pub struct BuildEngine {
    config: BuildConfig,
    cache: super::BuildCache,
}

impl BuildEngine {
    /// Create a new build engine
    pub fn new(config: BuildConfig) -> Self {
        let cache = super::BuildCache::new(config.cache_dir.clone());
        Self { config, cache }
    }

    /// Build a manifest for a given target
    pub async fn build(&self, manifest: &Manifest, target: &str) -> CadiResult<BuildResult> {
        let start = std::time::Instant::now();
        
        let target_config = manifest.find_target(target)
            .ok_or_else(|| CadiError::BuildFailed(format!("Target '{}' not found", target)))?;
        
        tracing::info!("Building target '{}' for platform '{}'", target, target_config.platform);
        
        // Create build plan
        let plan = super::BuildPlan::from_manifest(manifest, target)?;
        
        if self.config.verbose {
            tracing::debug!("Build plan: {} steps", plan.steps.len());
        }
        
        let mut built = Vec::new();
        let mut cached = Vec::new();
        let mut failed = Vec::new();
        
        // Execute build plan
        for step in &plan.steps {
            // Check cache first
            if let Some(chunk_id) = &step.chunk_id {
                if self.cache.has(chunk_id)? {
                    if self.config.verbose {
                        tracing::info!("Cache hit for {}", chunk_id);
                    }
                    println!("  {} Fetched {} from cache", 
                        console::style("✓").green(),
                        console::style(&step.name).cyan());
                    cached.push(chunk_id.clone());
                    continue;
                }
            }
            
            // Execute transformation
            println!("  {} Building {}...", 
                console::style("→").cyan(),
                console::style(&step.name).yellow());
            
            match self.execute_step(step).await {
                Ok(chunk_id) => {
                    tracing::debug!("Built {}", chunk_id);
                    built.push(chunk_id);
                }
                Err(e) => {
                    let failure = BuildFailure {
                        chunk_id: step.chunk_id.clone().unwrap_or_else(|| step.name.clone()),
                        error: e.to_string(),
                    };
                    failed.push(failure);
                    
                    if self.config.fail_fast {
                        break;
                    }
                }
            }
        }
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        Ok(BuildResult {
            built,
            cached,
            failed,
            duration_ms,
        })
    }

    /// Execute a single build step
    async fn execute_step(&self, step: &super::BuildStep) -> CadiResult<String> {
        tracing::info!("Executing step: {}", step.name);
        
        // Prepare inputs with paths
        let mut prepared_inputs = Vec::new();
        for input in &step.inputs {
            let mut prepared = input.clone();
            if self.cache.has(&input.chunk_id)? {
                prepared.path = Some(self.cache.get_path(&input.chunk_id).to_string_lossy().to_string());
            }
            prepared_inputs.push(prepared);
        }

        // Execute the transformation
        let transformer = super::Transformer::new();
        let result = transformer.transform(&step.transform, &prepared_inputs).await?;
        
        // Store in cache
        if let Some(ref chunk_id) = step.chunk_id {
            self.cache.store(chunk_id, &result)?;
        }
        
        Ok(step.chunk_id.clone().unwrap_or_else(|| step.name.clone()))
    }

    /// Get the path to a cached chunk
    pub fn get_chunk_path(&self, chunk_id: &str) -> Option<PathBuf> {
        if self.cache.has(chunk_id).unwrap_or(false) {
            Some(self.cache.get_path(chunk_id))
        } else {
            None
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CadiResult<CacheStats> {
        self.cache.stats()
    }
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub hit_rate: f64,
}

// Placeholder for num_cpus - in actual impl would use num_cpus crate
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)
    }
}

// Placeholder for dirs - in actual impl would use directories crate
mod dirs {
    use std::path::PathBuf;
    
    pub fn cache_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache"))
    }
}
