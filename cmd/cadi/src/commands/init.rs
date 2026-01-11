use anyhow::Result;
use clap::Args;
use console::style;
use dialoguer::Confirm;
use std::path::PathBuf;

use crate::config::{self, CadiConfig};

/// Arguments for the init command
#[derive(Args)]
pub struct InitArgs {
    /// Registry URL to use
    #[arg(short, long)]
    registry: Option<String>,

    /// Generate a new signing key
    #[arg(long)]
    generate_key: bool,

    /// Force overwrite existing configuration
    #[arg(short, long)]
    force: bool,
}

/// Execute the init command
pub async fn execute(args: InitArgs, _config: &CadiConfig) -> Result<()> {
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
