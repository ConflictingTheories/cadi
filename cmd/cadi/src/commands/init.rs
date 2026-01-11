use anyhow::Result;
use clap::Args;
use console::style;
use dialoguer::Confirm;
use std::path::PathBuf;

use crate::config::{self, CadiConfig};

/// Arguments for the init command
#[derive(Args)]
pub struct InitArgs {
    /// Directory to initialize (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Project name (defaults to directory name)
    #[arg(short, long)]
    name: Option<String>,

    /// Registry URL to use
    #[arg(short, long)]
    registry: Option<String>,

    /// Template to use (minimal, library, application)
    #[arg(short, long, default_value = "minimal")]
    template: String,

    /// Generate a new signing key
    #[arg(long)]
    generate_key: bool,

    /// Force overwrite existing configuration
    #[arg(short, long)]
    force: bool,

    /// Initialize global config only (not a project)
    #[arg(long)]
    global: bool,
}

/// Execute the init command
pub async fn execute(args: InitArgs, _config: &CadiConfig) -> Result<()> {
    if args.global {
        return init_global(args).await;
    }
    
    init_project(args).await
}

/// Initialize a CADI project in the specified directory
async fn init_project(args: InitArgs) -> Result<()> {
    let project_dir = if args.path.is_absolute() {
        args.path.clone()
    } else {
        std::env::current_dir()?.join(&args.path)
    };
    
    // Create project directory if it doesn't exist
    std::fs::create_dir_all(&project_dir)?;
    
    let project_name = args.name
        .or_else(|| project_dir.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string());
    
    let manifest_path = project_dir.join("cadi.yaml");
    let cadi_dir = project_dir.join(".cadi");
    
    // Check if already initialized
    if manifest_path.exists() && !args.force {
        let overwrite = Confirm::new()
            .with_prompt("Project already initialized. Overwrite?")
            .default(false)
            .interact()?;
        
        if !overwrite {
            println!("{}", style("Initialization cancelled.").yellow());
            return Ok(());
        }
    }

    println!("{}", style(format!("Initializing CADI project: {}", project_name)).bold());

    // Create .cadi directory
    std::fs::create_dir_all(&cadi_dir)?;
    println!("  {} Created .cadi directory", style("✓").green());

    // Create .cadi/cache directory
    let cache_dir = cadi_dir.join("cache");
    std::fs::create_dir_all(&cache_dir)?;
    
    // Create local repos.cfg
    let repos_cfg = create_repos_cfg(&args.registry)?;
    std::fs::write(cadi_dir.join("repos.cfg"), repos_cfg)?;
    println!("  {} Created .cadi/repos.cfg", style("✓").green());

    // Create cadi.yaml manifest
    let manifest = create_manifest(&project_name, &args.template)?;
    std::fs::write(&manifest_path, manifest)?;
    println!("  {} Created cadi.yaml", style("✓").green());

    // Create src directory with example based on template
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;
    
    match args.template.as_str() {
        "library" => {
            std::fs::write(src_dir.join("lib.rs"), TEMPLATE_LIB_RS)?;
            println!("  {} Created src/lib.rs", style("✓").green());
        }
        "application" => {
            std::fs::write(src_dir.join("main.rs"), TEMPLATE_MAIN_RS)?;
            println!("  {} Created src/main.rs", style("✓").green());
        }
        _ => {
            std::fs::write(src_dir.join("lib.rs"), TEMPLATE_MINIMAL_RS)?;
            println!("  {} Created src/lib.rs", style("✓").green());
        }
    }

    // Generate signing key if requested
    if args.generate_key {
        let key_path = cadi_dir.join("signing.key");
        generate_signing_key(&key_path)?;
        println!("  {} Generated signing key", style("✓").green());
    }

    println!();
    println!("{}", style("Project initialized successfully!").green().bold());
    println!();
    println!("Project: {}", project_name);
    println!("Location: {}", project_dir.display());
    println!();
    println!("Next steps:");
    println!("  {} Import source code:", style("1.").cyan());
    println!("     cadi import ./src --language rust");
    println!();
    println!("  {} Build the project:", style("2.").cyan());
    println!("     cadi build --target default");

    Ok(())
}

/// Initialize global CADI configuration
async fn init_global(args: InitArgs) -> Result<()> {
    let config_path = config::config_file_path();
    
    // Check if config already exists
    if config_path.exists() && !args.force {
        let overwrite = Confirm::new()
            .with_prompt("Configuration already exists. Overwrite?")
            .default(false)
            .interact()?;
        
        if !overwrite {
            println!("{}", style("Initialization cancelled.").yellow());
            return Ok(());
        }
    }

    println!("{}", style("Initializing CADI...").bold());

    // Create configuration directory
    let config_dir = config::config_dir();
    std::fs::create_dir_all(&config_dir)?;
    println!("  {} Created config directory: {}", style("✓").green(), config_dir.display());

    // Create cache directory
    let mut new_config = CadiConfig::default();
    
    // Set registry URL if provided
    if let Some(registry) = args.registry {
        new_config.registry.url = registry;
    }

    // Create cache directory
    std::fs::create_dir_all(&new_config.cache.dir)?;
    println!("  {} Created cache directory: {}", style("✓").green(), new_config.cache.dir.display());

    // Create store subdirectories
    let chunks_dir = new_config.cache.dir.join("chunks");
    let blobs_dir = new_config.cache.dir.join("blobs").join("sha256");
    std::fs::create_dir_all(&chunks_dir)?;
    std::fs::create_dir_all(&blobs_dir)?;
    println!("  {} Created store directories", style("✓").green());

    // Generate signing key if requested
    if args.generate_key {
        let key_path = config_dir.join("signing.key");
        generate_signing_key(&key_path)?;
        new_config.security.signing_key = Some(key_path.clone());
        println!("  {} Generated signing key: {}", style("✓").green(), key_path.display());
    }

    // Save configuration
    config::save_config(&new_config, None)?;
    println!("  {} Created config file: {}", style("✓").green(), config_path.display());

    println!();
    println!("{}", style("CADI initialized successfully!").green().bold());
    println!();
    println!("Configuration:");
    println!("  Registry: {}", new_config.registry.url);
    println!("  Cache: {}", new_config.cache.dir.display());
    if let Some(key_path) = &new_config.security.signing_key {
        println!("  Signing key: {}", key_path.display());
    }
    println!();
    println!("Next steps:");
    println!("  {} Import a project:", style("1.").cyan());
    println!("     cadi import ./my-project");
    println!();
    println!("  {} Build from manifest:", style("2.").cyan());
    println!("     cadi build manifest.cadi.yaml");

    Ok(())
}

/// Generate an Ed25519 signing key
fn generate_signing_key(path: &PathBuf) -> Result<()> {
    use sha2::{Sha256, Digest};
    use std::io::Write;

    // Generate random bytes for the key (simplified - real impl would use ed25519-dalek)
    let mut hasher = Sha256::new();
    hasher.update(uuid::Uuid::new_v4().to_string().as_bytes());
    hasher.update(chrono::Utc::now().to_rfc3339().as_bytes());
    let seed = hasher.finalize();

    // Write key file (PEM format placeholder)
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "-----BEGIN CADI PRIVATE KEY-----")?;
    writeln!(file, "{}", base64_encode(&seed))?;
    writeln!(file, "-----END CADI PRIVATE KEY-----")?;

    // Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        
        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        
        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }
    
    result
}

/// Create repos.cfg content
fn create_repos_cfg(registry_url: &Option<String>) -> Result<String> {
    let url = registry_url.as_deref().unwrap_or("http://localhost:8080");
    Ok(format!(r#"# CADI Repository Configuration

[registries]
default = "local"

[registries.local]
name = "Local Development Registry"
url = "{}"
priority = 0
trust_level = "full"
enabled = true
capabilities = ["search", "fetch", "push"]

[registries.official]
name = "CADI Official Registry"
url = "https://registry.cadi.dev"
priority = 10
trust_level = "verified"
enabled = false
capabilities = ["search", "fetch"]

[cache]
directory = ".cadi/cache"
max_size_mb = 1024

[security]
verify_signatures = false
allow_unsigned = true
"#, url))
}

/// Create cadi.yaml manifest content
fn create_manifest(name: &str, template: &str) -> Result<String> {
    let description = match template {
        "library" => "A CADI library project",
        "application" => "A CADI application",
        _ => "A CADI project",
    };
    
    Ok(format!(r#"# CADI Manifest
manifest_id: "{name}-manifest"
manifest_version: "1.0"

application:
  name: "{name}"
  description: "{description}"
  version: "0.1.0"
  license: "MIT"

build_graph:
  nodes:
    - id: "{name}-core"
      source_cadi: null  # Will be set after import
      representations: []
      
  edges: []

build_targets:
  - name: "default"
    platform: "native"
    nodes:
      - id: "{name}-core"
    bundle:
      format: "executable"
      output: "dist/{name}"

dependencies:
  resolution_strategy: "newest"
"#))
}

// Template file contents
const TEMPLATE_MINIMAL_RS: &str = r#"//! Minimal CADI project

/// Hello world function
pub fn hello() -> &'static str {
    "Hello from CADI!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from CADI!");
    }
}
"#;

const TEMPLATE_LIB_RS: &str = r#"//! CADI Library
//!
//! This is a CADI library project.

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main library entry point
pub fn init() {
    println!("Library initialized");
}

/// Example function
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#;

const TEMPLATE_MAIN_RS: &str = r#"//! CADI Application
//!
//! This is a CADI application project.

fn main() {
    println!("Hello from CADI application!");
    
    // Your application logic here
    run();
}

fn run() {
    println!("Application running...");
}
"#;
