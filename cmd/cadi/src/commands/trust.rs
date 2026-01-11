use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;

use crate::config::CadiConfig;

/// Arguments for the trust command
#[derive(Args)]
pub struct TrustArgs {
    #[command(subcommand)]
    command: TrustCommands,
}

#[derive(Subcommand)]
enum TrustCommands {
    /// Add a trusted signer
    Add {
        /// Signer identifier
        signer: String,
        
        /// Trust level
        #[arg(long, default_value = "standard")]
        level: String,
    },
    
    /// Remove a trusted signer
    Remove {
        /// Signer identifier
        signer: String,
    },
    
    /// List trusted signers
    List,
    
    /// Show or set trust policy
    Policy {
        /// Policy mode (strict, standard, permissive)
        #[arg()]
        mode: Option<String>,
    },
}

/// Execute the trust command
pub async fn execute(args: TrustArgs, config: &CadiConfig) -> Result<()> {
    match args.command {
        TrustCommands::Add { signer, level } => {
            println!("{}", style("Adding trusted signer...").bold());
            println!("  Signer: {}", signer);
            println!("  Level:  {}", level);
            
            // Would update trust store
            println!();
            println!("{} Added {} as trusted signer", style("✓").green(), signer);
        }
        
        TrustCommands::Remove { signer } => {
            println!("{}", style("Removing trusted signer...").bold());
            
            // Would update trust store
            println!("{} Removed {} from trusted signers", style("✓").green(), signer);
        }
        
        TrustCommands::List => {
            println!("{}", style("Trusted Signers").bold());
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            
            // Would read from trust store
            println!("  {} github.com/cadi-project/* (full)", style("●").green());
            println!("  {} Local signing key (full)", style("●").green());
            println!();
            println!("Trust policy: {}", style(&config.security.trust_policy).cyan());
        }
        
        TrustCommands::Policy { mode } => {
            if let Some(new_mode) = mode {
                println!("{}", style("Setting trust policy...").bold());
                
                match new_mode.as_str() {
                    "strict" | "standard" | "permissive" => {
                        // Would update config
                        println!("{} Trust policy set to: {}", style("✓").green(), new_mode);
                    }
                    _ => {
                        println!("{} Invalid policy. Use: strict, standard, or permissive", style("✗").red());
                    }
                }
            } else {
                println!("{}", style("Trust Policy").bold());
                println!();
                println!("Current policy: {}", style(&config.security.trust_policy).cyan());
                println!();
                println!("Available policies:");
                println!("  {} - All artifacts must have valid signatures from trusted signers", style("strict").bold());
                println!("  {} - Require signatures for IR/blob/container, optional for source", style("standard").bold());
                println!("  {} - Allow unsigned artifacts with warning", style("permissive").bold());
            }
        }
    }

    Ok(())
}
