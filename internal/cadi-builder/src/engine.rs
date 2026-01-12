//! Build engine for CADI

use cadi_core::{CadiError, CadiResult, Manifest};
use cadi_registry::client::RegistryClient;
use serde_json::Value as JsonValue;
use base64::engine::general_purpose;
use base64::Engine as _;
use ed25519_dalek::Verifier;
use ed25519_dalek::PublicKey;
use ed25519_dalek::Signature;
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
    /// Require artifacts to be signed / attested before materialization
    pub require_signed: bool,
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
            require_signed: false,
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
        let mut plan = super::BuildPlan::from_manifest(manifest, target)?;
        
        if self.config.verbose {
            tracing::debug!("Build plan: {} steps", plan.steps.len());
        }

        // Apply materialization preferences (stub - selection logic to be implemented)
        self.apply_materialization_preferences(&mut plan, target_config);

        // Optional trust verification before executing plan
        if self.config.require_signed {
            if let Err(e) = self.verify_trust(&plan, manifest, target_config).await {
                return Err(e);
            }
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

    /// Apply materialization preferences to the build plan.
    ///
    /// This is a placeholder for selection logic that will choose representations
    /// (binary vs source vs ir vs embed) per node according to manifest hints and
    /// build target preferences.
    fn apply_materialization_preferences(&self, _plan: &mut super::BuildPlan, _target: &cadi_core::BuildTarget) {
        tracing::debug!("apply_materialization_preferences: stub - no-op");
        // TODO: implement selection algorithm that rewrites plan.steps to prefer
        // chosen representations and inject transforms where necessary.
    }

    /// Verify trust/attestations for the planned artifacts. Currently a stub.
    async fn verify_trust(&self, _plan: &super::BuildPlan, manifest: &cadi_core::Manifest, target: &cadi_core::BuildTarget) -> CadiResult<()> {
        tracing::info!("verify_trust: checking attestations require_signed={}", self.config.require_signed);

        // Determine effective trust requirements: target overrides manifest defaults
        let mut effective_req = manifest.trust_defaults.clone();
        if let Some(treq) = &target.trust_requirements {
            // merge: target fields take precedence when set
            if effective_req.is_none() {
                effective_req = Some(treq.clone());
            } else if let Some(er) = effective_req.as_mut() {
                if treq.minimum_signatures.is_some() { er.minimum_signatures = treq.minimum_signatures; }
                if !treq.required_attestation_types.is_empty() { er.required_attestation_types = treq.required_attestation_types.clone(); }
                if treq.required_signers.is_some() { er.required_signers = treq.required_signers.clone(); }
                if treq.max_age_days.is_some() { er.max_age_days = treq.max_age_days; }
                er.transitive_policy = treq.transitive_policy.clone().or(er.transitive_policy.clone());
            }
        }

        let client = RegistryClient::default_client()?;

        for step in &_plan.steps {
            if let Some(chunk_id) = &step.chunk_id {
                tracing::debug!("verify_trust: fetching metadata for {}", chunk_id);
                let chunk = client.fetch_chunk_meta(chunk_id).await?;

                // Determine minimum signatures
                let min_sigs = effective_req.as_ref().and_then(|r| r.minimum_signatures).unwrap_or(1) as usize;

                if chunk.signatures.len() < min_sigs {
                    return Err(CadiError::TrustPolicyViolation(format!("Chunk {} has {} signatures, requires {}", chunk_id, chunk.signatures.len(), min_sigs)));
                }

                // Check build receipt if present for attestation details
                if let Some(build_receipt_id) = &chunk.lineage.build_receipt {
                    tracing::debug!("verify_trust: fetching build receipt {}", build_receipt_id);
                    match client.fetch_chunk(build_receipt_id).await {
                        Ok(bytes) => {
                            if let Ok(json) = serde_json::from_slice::<JsonValue>(&bytes) {
                                // If target requested specific attestation types, ensure presence
                                if let Some(req) = &effective_req {
                                    for atype in &req.required_attestation_types {
                                        let present = json.pointer(&format!("/attestation/{}", atype)).is_some() || json.pointer(&format!("/{}",&atype)).is_some();
                                        if !present {
                                            return Err(CadiError::TrustPolicyViolation(format!("Build receipt {} missing required attestation type {}", build_receipt_id, atype)));
                                        }
                                    }

                                    // Required signers: try to find signer id in receipt
                                    if let Some(required_signers) = &req.required_signers {
                                        let mut found = false;
                                        if let Some(signer_id) = json.pointer("/signer/id").and_then(|v| v.as_str()) {
                                            if required_signers.iter().any(|rs| rs == signer_id) {
                                                found = true;
                                            }
                                        }
                                        if !found {
                                            tracing::warn!("Required signer not found in build receipt {}: {:?}", build_receipt_id, required_signers);
                                            return Err(CadiError::TrustPolicyViolation(format!("Build receipt {} missing required signer", build_receipt_id)));
                                        }
                                    }

                                    // Max age check
                                    if let Some(max_days) = req.max_age_days {
                                        if let Some(ts) = json.pointer("/signature/timestamp").and_then(|v| v.as_str()) {
                                            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(ts) {
                                                let age = chrono::Utc::now().signed_duration_since(parsed.with_timezone(&chrono::Utc));
                                                if age.num_days() > max_days as i64 {
                                                    return Err(CadiError::TrustPolicyViolation(format!("Build receipt {} signature too old ({} days > {})", build_receipt_id, age.num_days(), max_days)));
                                                }
                                            }
                                        }
                                    }

                                    // Cryptographic verification: attempt to verify ed25519 signatures if present
                                    if let Err(e) = Self::verify_attestation_signature(&json, &bytes) {
                                        return Err(e);
                                    }
                                }
                            } else {
                                tracing::warn!("Build receipt {} is not valid JSON", build_receipt_id);
                                return Err(CadiError::VerificationFailed(format!("Invalid build receipt {}", build_receipt_id)));
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to fetch build receipt {}: {}", build_receipt_id, e);
                            return Err(e);
                        }
                    }
                } else {
                    tracing::warn!("Chunk {} missing build_receipt lineage", chunk_id);
                    // If policy requires attestations, fail
                    if let Some(req) = &effective_req {
                        if !req.required_attestation_types.is_empty() || req.minimum_signatures.is_some() {
                            return Err(CadiError::TrustPolicyViolation(format!("Chunk {} missing build_receipt required by policy", chunk_id)));
                        }
                    }
                }
            }
        }

        Ok(())
    }

/// Verify attestation signature(s) inside a build receipt JSON blob.
/// Attempts to find common fields and verify ed25519 signatures when possible.
fn verify_attestation_signature(json: &JsonValue, _raw_bytes: &[u8]) -> CadiResult<()> {
    // Look for signature value and signer public key in JSON paths
    let sig_val_opt = json.pointer("/attestation/signature/value")
        .or_else(|| json.pointer("/signature/value"))
        .and_then(|v| v.as_str());

    let pk_opt = json.pointer("/signer/identity/key/public_key")
        .and_then(|v| v.as_str());

    let alg_opt = json.pointer("/attestation/signature/algorithm")
        .or_else(|| json.pointer("/signature/algorithm"))
        .and_then(|v| v.as_str());

            if let (Some(sig_b64), Some(pub_b64), Some(alg)) = (sig_val_opt, pk_opt, alg_opt) {
        if alg.to_lowercase().contains("ed25519") {
            // decode base64
            let sig_bytes = general_purpose::STANDARD.decode(sig_b64)
                .map_err(|e| CadiError::SignatureInvalid(format!("Invalid signature base64: {}", e)))?;
            let pk_bytes = general_purpose::STANDARD.decode(pub_b64)
                .map_err(|e| CadiError::SignatureInvalid(format!("Invalid pubkey base64: {}", e)))?;

            let pk = PublicKey::from_bytes(&pk_bytes)
                .map_err(|e| CadiError::SignatureInvalid(format!("Invalid public key: {}", e)))?;
            let sig = Signature::from_bytes(&sig_bytes)
                .map_err(|e| CadiError::SignatureInvalid(format!("Invalid signature bytes: {}", e)))?;

            // Verify signature over the signed payload: remove signature field then serialize
            let mut copy = json.clone();
            if let Some(att) = copy.get_mut("attestation") {
                if att.is_object() {
                    att.as_object_mut().unwrap().remove("signature");
                }
            }
            let signed_bytes = serde_json::to_vec(&copy)
                .map_err(|e| CadiError::VerificationFailed(e.to_string()))?;

            pk.verify(&signed_bytes, &sig)
                .map_err(|e| CadiError::SignatureInvalid(format!("Signature verification failed: {}", e)))?;
        } else {
            tracing::info!("Unsupported signature algorithm {} - skipping cryptographic verification", alg);
        }
    } else {
        tracing::debug!("No direct signature/pubkey pair found in receipt JSON to verify");
    }

    Ok(())
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
