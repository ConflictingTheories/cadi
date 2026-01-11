use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CADI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CadiConfig {
    /// Registry configuration
    #[serde(default)]
    pub registry: RegistryConfig,

    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,

    /// LLM optimization configuration
    #[serde(default)]
    pub llm: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Registry URL
    #[serde(default = "default_registry_url")]
    pub url: String,

    /// Default namespace
    #[serde(default)]
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication token
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache directory
    #[serde(default = "default_cache_dir")]
    pub dir: PathBuf,

    /// Maximum cache size in GB
    #[serde(default = "default_max_size_gb")]
    pub max_size_gb: u64,

    /// Eviction policy
    #[serde(default = "default_eviction_policy")]
    pub eviction_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Build parallelism
    #[serde(default = "default_parallelism")]
    pub parallelism: usize,

    /// Preferred representations (in order)
    #[serde(default = "default_prefer_representation")]
    pub prefer_representation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Path to signing key
    #[serde(default)]
    pub signing_key: Option<PathBuf>,

    /// Trust policy mode
    #[serde(default = "default_trust_policy")]
    pub trust_policy: String,

    /// Verify signatures on fetch
    #[serde(default = "default_true")]
    pub verify_on_fetch: bool,

    /// Sandbox untrusted code
    #[serde(default = "default_true")]
    pub sandbox_untrusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Embedding model
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,

    /// Maximum tokens for summaries
    #[serde(default = "default_summary_max_tokens")]
    pub summary_max_tokens: usize,
}

// Default value functions
fn default_registry_url() -> String {
    "https://registry.cadi.dev".to_string()
}

fn default_cache_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "cadi", "cadi") {
        proj_dirs.cache_dir().to_path_buf()
    } else {
        PathBuf::from("~/.cadi/store")
    }
}

fn default_max_size_gb() -> u64 {
    10
}

fn default_eviction_policy() -> String {
    "lru".to_string()
}

fn default_parallelism() -> usize {
    num_cpus::get().max(1)
}

fn default_prefer_representation() -> Vec<String> {
    vec!["binary".to_string(), "wasm".to_string(), "source".to_string()]
}

fn default_trust_policy() -> String {
    "standard".to_string()
}

fn default_true() -> bool {
    true
}

fn default_embedding_model() -> String {
    "text-embedding-3-large".to_string()
}

fn default_summary_max_tokens() -> usize {
    500
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: default_registry_url(),
            namespace: None,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self { token: None }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            dir: default_cache_dir(),
            max_size_gb: default_max_size_gb(),
            eviction_policy: default_eviction_policy(),
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            parallelism: default_parallelism(),
            prefer_representation: default_prefer_representation(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            signing_key: None,
            trust_policy: default_trust_policy(),
            verify_on_fetch: true,
            sandbox_untrusted: true,
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            embedding_model: default_embedding_model(),
            summary_max_tokens: default_summary_max_tokens(),
        }
    }
}

impl Default for CadiConfig {
    fn default() -> Self {
        Self {
            registry: RegistryConfig::default(),
            auth: AuthConfig::default(),
            cache: CacheConfig::default(),
            build: BuildConfig::default(),
            security: SecurityConfig::default(),
            llm: LlmConfig::default(),
        }
    }
}

/// Get the default configuration directory
pub fn config_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "cadi", "cadi") {
        proj_dirs.config_dir().to_path_buf()
    } else {
        PathBuf::from("~/.cadi")
    }
}

/// Get the default configuration file path
pub fn config_file_path() -> PathBuf {
    config_dir().join("config.yaml")
}

/// Find project-local config (looks for .cadi/repos.cfg or cadi.yaml in current/parent dirs)
pub fn find_project_config() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        // Check for .cadi/repos.cfg
        let repos_cfg = current.join(".cadi").join("repos.cfg");
        if repos_cfg.exists() {
            return Some(repos_cfg);
        }
        // Check for cadi.yaml manifest
        let manifest = current.join("cadi.yaml");
        if manifest.exists() {
            return Some(current.join(".cadi").join("repos.cfg"));
        }
        // Go up to parent
        if !current.pop() {
            break;
        }
    }
    None
}

/// Load configuration from file or use defaults
/// Priority: explicit path > project-local .cadi/repos.cfg > user config > defaults
pub fn load_config(path: Option<&str>) -> Result<CadiConfig> {
    // If explicit path provided, use it
    if let Some(p) = path {
        let config_path = PathBuf::from(p);
        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let config: CadiConfig = serde_yaml::from_str(&contents)?;
            return Ok(config);
        }
    }

    // Try project-local config
    if let Some(project_config) = find_project_config() {
        if project_config.exists() {
            // For now, just check if it exists; could parse TOML repos.cfg later
            tracing::debug!("Found project config: {:?}", project_config);
        }
    }

    // Try user-level config
    let config_path = config_file_path();
    if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path)?;
        let config: CadiConfig = serde_yaml::from_str(&contents)?;
        return Ok(config);
    }

    // Return defaults
    Ok(CadiConfig::default())
}

/// Load configuration with local development settings
#[allow(dead_code)]
pub fn load_local_config() -> CadiConfig {
    CadiConfig {
        registry: RegistryConfig {
            url: "http://localhost:8080".to_string(),
            namespace: None,
        },
        auth: AuthConfig::default(),
        cache: CacheConfig::default(),
        build: BuildConfig::default(),
        security: SecurityConfig {
            signing_key: None,
            trust_policy: "permissive".to_string(),
            verify_on_fetch: false,
            sandbox_untrusted: false,
        },
        llm: LlmConfig::default(),
    }
}

/// Save configuration to file
pub fn save_config(config: &CadiConfig, path: Option<&str>) -> Result<()> {
    let config_path = path
        .map(PathBuf::from)
        .unwrap_or_else(config_file_path);

    // Create parent directories if needed
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = serde_yaml::to_string(config)?;
    std::fs::write(&config_path, contents)?;
    
    Ok(())
}

// Add num_cpus dependency for parallelism default
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
