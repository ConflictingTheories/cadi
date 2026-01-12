use super::super::config::CadiConfig as Config;
use anyhow::{Context, Result};
use clap::Args;
use std::fs;
use std::path::PathBuf;
use cadi_core::parser::parse_file;
use cadi_core::validator::Validator;

/// Validate a CADL file against the specification
#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the CADL file to validate
    #[arg(required = true)]
    pub file: PathBuf,

    /// Verbose output showing parsed structure
    #[arg(short, long)]
    pub verbose: bool,
}

pub async fn execute(args: ValidateArgs, _config: &Config) -> Result<()> {
    println!("Validating CADL file: {:?}", args.file);

    let content = fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read file: {:?}", args.file))?;

    // Phase 1: Parsing
    println!("Phase 1: Parsing...");
    let doc = match parse_file(&content) {
        Ok(doc) => {
            println!("✓ Parsing successful");
            doc
        }
        Err(e) => {
            eprintln!("✕ Parsing failed:\n{}", e);
            std::process::exit(1);
        }
    };

    if args.verbose {
        println!("{:#?}", doc);
    }

    // Phase 2: Semantic Validation
    println!("Phase 2: Semantic Validation...");
    let validator = Validator::new();
    match validator.validate(&doc) {
        Ok(_) => {
            println!("✓ Validation successful");
            println!("File {:?} is valid CADL v2.", args.file);
        }
        Err(errors) => {
            eprintln!("✕ Validation failed with {} errors:", errors.len());
            for (i, err) in errors.iter().enumerate() {
                eprintln!("  {}. [{}] {}", i + 1, err.path, err.message);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}
