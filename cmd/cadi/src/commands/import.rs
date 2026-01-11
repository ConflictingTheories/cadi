use anyhow::{Context, Result};
use clap::Args;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::CadiConfig;

/// Arguments for the import command
#[derive(Args)]
pub struct ImportArgs {
    /// Path to the project to import
    #[arg(required = true)]
    path: PathBuf,

    /// Output manifest path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Don't publish to registry (local only)
    #[arg(long)]
    no_publish: bool,

    /// Project name override
    #[arg(short, long)]
    name: Option<String>,
}

/// Detected project type
#[derive(Debug, Clone)]
enum ProjectType {
    TypeScript,
    JavaScript,
    Rust,
    C,
    Python,
    Go,
    Unknown,
}

/// Execute the import command
pub async fn execute(args: ImportArgs, config: &CadiConfig) -> Result<()> {
    let path = args.path.canonicalize()
        .context("Failed to resolve project path")?;
    
    println!("{}", style("Importing project...").bold());
    println!("  Path: {}", path.display());

    // Detect project type
    let project_type = detect_project_type(&path)?;
    println!("  {} Detected project type: {:?}", style("✓").green(), project_type);

    // Get project name
    let project_name = args.name.clone()
        .or_else(|| path.file_name().and_then(|n| n.to_str()).map(String::from))
        .unwrap_or_else(|| "unnamed".to_string());
    println!("  Project name: {}", project_name);

    // Collect source files
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    spinner.set_message("Scanning source files...");

    let files = collect_source_files(&path, &project_type)?;
    spinner.finish_with_message(format!("Found {} source files", files.len()));

    // Hash files and create chunk
    let progress = ProgressBar::new(files.len() as u64);
    progress.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{wide_bar:.cyan/blue} {pos}/{len}")
        .unwrap());
    progress.set_message("Hashing files...");

    let mut file_entries = Vec::new();
    let mut content_hasher = Sha256::new();

    for file_path in &files {
        let relative_path = file_path.strip_prefix(&path)?;
        let content = std::fs::read(file_path)?;
        
        // Hash file content
        let mut file_hasher = Sha256::new();
        file_hasher.update(&content);
        let file_hash = format!("sha256:{:x}", file_hasher.finalize());

        // Include in chunk content hash
        content_hasher.update(relative_path.to_string_lossy().as_bytes());
        content_hasher.update(&content);

        file_entries.push(FileEntry {
            path: relative_path.to_string_lossy().to_string(),
            hash: file_hash,
            size: content.len(),
        });

        progress.inc(1);
    }

    progress.finish_with_message("Files hashed");

    // Generate chunk ID
    let chunk_hash = format!("{:x}", content_hasher.finalize());
    let chunk_id = format!("chunk:sha256:{}", chunk_hash);
    println!("  {} Generated chunk ID: {}", style("✓").green(), &chunk_id[..40]);

    // Detect dependencies
    let dependencies = detect_dependencies(&path, &project_type)?;
    if !dependencies.is_empty() {
        println!("  {} Detected {} dependencies", style("✓").green(), dependencies.len());
    }

    // Detect entrypoints
    let entrypoints = detect_entrypoints(&path, &project_type)?;
    if !entrypoints.is_empty() {
        println!("  {} Detected {} entrypoints", style("✓").green(), entrypoints.len());
    }

    // Create Source CADI spec
    let source_cadi = create_source_cadi(
        &chunk_id,
        &project_name,
        &project_type,
        file_entries,
        dependencies,
        entrypoints,
    )?;

    // Save chunk to local store
    let chunks_dir = config.cache.dir.join("chunks");
    std::fs::create_dir_all(&chunks_dir)?;
    
    let chunk_file = chunks_dir.join(format!("{}.json", chunk_hash));
    std::fs::write(&chunk_file, serde_json::to_string_pretty(&source_cadi)?)?;
    println!("  {} Saved chunk to local store", style("✓").green());

    // Create manifest
    let manifest = create_manifest(&project_name, &chunk_id)?;
    
    let output_path = args.output.unwrap_or_else(|| {
        path.join(format!("{}.cadi.yaml", project_name.to_lowercase().replace(" ", "-")))
    });
    
    std::fs::write(&output_path, serde_yaml::to_string(&manifest)?)?;
    println!("  {} Created manifest: {}", style("✓").green(), output_path.display());

    println!();
    println!("{}", style("Import complete!").green().bold());
    println!();
    println!("Chunk ID: {}", chunk_id);
    println!("Manifest: {}", output_path.display());
    println!();
    println!("Next steps:");
    println!("  {} Build the project:", style("1.").cyan());
    println!("     cadi build {}", output_path.display());
    if !args.no_publish {
        println!();
        println!("  {} Publish to registry:", style("2.").cyan());
        println!("     cadi publish");
    }

    Ok(())
}

#[derive(Debug)]
struct FileEntry {
    path: String,
    hash: String,
    size: usize,
}

fn detect_project_type(path: &Path) -> Result<ProjectType> {
    // Check for TypeScript
    if path.join("tsconfig.json").exists() {
        return Ok(ProjectType::TypeScript);
    }
    
    // Check for Rust
    if path.join("Cargo.toml").exists() {
        return Ok(ProjectType::Rust);
    }
    
    // Check for Node.js/JavaScript
    if path.join("package.json").exists() {
        return Ok(ProjectType::JavaScript);
    }
    
    // Check for C
    if path.join("Makefile").exists() || path.join("CMakeLists.txt").exists() {
        return Ok(ProjectType::C);
    }
    
    // Check for Python
    if path.join("setup.py").exists() || path.join("pyproject.toml").exists() {
        return Ok(ProjectType::Python);
    }
    
    // Check for Go
    if path.join("go.mod").exists() {
        return Ok(ProjectType::Go);
    }
    
    Ok(ProjectType::Unknown)
}

fn collect_source_files(path: &Path, project_type: &ProjectType) -> Result<Vec<PathBuf>> {
    let extensions: Vec<&str> = match project_type {
        ProjectType::TypeScript => vec!["ts", "tsx", "js", "jsx", "json"],
        ProjectType::JavaScript => vec!["js", "jsx", "mjs", "cjs", "json"],
        ProjectType::Rust => vec!["rs", "toml"],
        ProjectType::C => vec!["c", "h", "cpp", "hpp", "cc", "hh"],
        ProjectType::Python => vec!["py", "pyi"],
        ProjectType::Go => vec!["go", "mod", "sum"],
        ProjectType::Unknown => vec!["txt", "md", "json", "yaml", "yml"],
    };

    let mut files = Vec::new();
    collect_files_recursive(path, &extensions, &mut files)?;
    Ok(files)
}

fn collect_files_recursive(dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) -> Result<()> {
    let ignore_dirs = ["node_modules", "target", ".git", "__pycache__", "dist", "build", ".next"];
    
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            if !ignore_dirs.contains(&dir_name) && !dir_name.starts_with('.') {
                collect_files_recursive(&path, extensions, files)?;
            }
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.contains(&ext) {
                files.push(path);
            }
        }
    }
    
    Ok(())
}

fn detect_dependencies(path: &Path, project_type: &ProjectType) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();
    
    match project_type {
        ProjectType::TypeScript | ProjectType::JavaScript => {
            let package_json = path.join("package.json");
            if package_json.exists() {
                let content = std::fs::read_to_string(&package_json)?;
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_object()) {
                        for (name, version) in deps {
                            dependencies.push(Dependency {
                                id: format!("npm:{}@{}", name, version.as_str().unwrap_or("*")),
                                optional: false,
                            });
                        }
                    }
                }
            }
        }
        ProjectType::Rust => {
            // Parse Cargo.toml dependencies (simplified)
            let cargo_toml = path.join("Cargo.toml");
            if cargo_toml.exists() {
                let content = std::fs::read_to_string(&cargo_toml)?;
                // Simplified parsing - real impl would use toml crate
                for line in content.lines() {
                    if line.contains(" = ") && !line.starts_with('[') && !line.starts_with('#') {
                        let parts: Vec<&str> = line.split('=').collect();
                        if parts.len() >= 2 {
                            let name = parts[0].trim();
                            dependencies.push(Dependency {
                                id: format!("crates:{}", name),
                                optional: false,
                            });
                        }
                    }
                }
            }
        }
        _ => {}
    }
    
    Ok(dependencies)
}

#[derive(Debug)]
struct Dependency {
    id: String,
    optional: bool,
}

fn detect_entrypoints(path: &Path, project_type: &ProjectType) -> Result<Vec<Entrypoint>> {
    let mut entrypoints = Vec::new();
    
    match project_type {
        ProjectType::TypeScript | ProjectType::JavaScript => {
            // Check package.json for main/module
            let package_json = path.join("package.json");
            if package_json.exists() {
                let content = std::fs::read_to_string(&package_json)?;
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(main) = pkg.get("main").and_then(|m| m.as_str()) {
                        entrypoints.push(Entrypoint {
                            symbol: "main".to_string(),
                            path: main.to_string(),
                        });
                    }
                }
            }
            
            // Common entry files
            for entry in ["src/index.ts", "src/index.tsx", "index.ts", "index.js", "src/main.ts"] {
                if path.join(entry).exists() {
                    entrypoints.push(Entrypoint {
                        symbol: "default".to_string(),
                        path: entry.to_string(),
                    });
                    break;
                }
            }
        }
        ProjectType::Rust => {
            if path.join("src/main.rs").exists() {
                entrypoints.push(Entrypoint {
                    symbol: "main".to_string(),
                    path: "src/main.rs".to_string(),
                });
            }
            if path.join("src/lib.rs").exists() {
                entrypoints.push(Entrypoint {
                    symbol: "lib".to_string(),
                    path: "src/lib.rs".to_string(),
                });
            }
        }
        ProjectType::C => {
            for entry in ["main.c", "src/main.c"] {
                if path.join(entry).exists() {
                    entrypoints.push(Entrypoint {
                        symbol: "main".to_string(),
                        path: entry.to_string(),
                    });
                    break;
                }
            }
        }
        _ => {}
    }
    
    Ok(entrypoints)
}

#[derive(Debug)]
struct Entrypoint {
    symbol: String,
    path: String,
}

fn create_source_cadi(
    chunk_id: &str,
    name: &str,
    project_type: &ProjectType,
    files: Vec<FileEntry>,
    dependencies: Vec<Dependency>,
    entrypoints: Vec<Entrypoint>,
) -> Result<serde_json::Value> {
    let language = match project_type {
        ProjectType::TypeScript => "typescript",
        ProjectType::JavaScript => "javascript",
        ProjectType::Rust => "rust",
        ProjectType::C => "c",
        ProjectType::Python => "python",
        ProjectType::Go => "go",
        ProjectType::Unknown => "unknown",
    };

    let source_cadi = serde_json::json!({
        "chunk_id": chunk_id,
        "cadi_type": "source",
        "meta": {
            "name": name,
            "description": format!("{} source code", name),
            "version": null,
            "tags": [language],
            "created_at": chrono::Utc::now().to_rfc3339(),
        },
        "provides": {
            "concepts": [],
            "interfaces": [],
            "abi": null
        },
        "licensing": {
            "license": "MIT",
            "restrictions": []
        },
        "lineage": {
            "parents": [],
            "build_receipt": null
        },
        "source": {
            "language": language,
            "version": null,
            "dialect": null,
            "files": files.iter().map(|f| serde_json::json!({
                "path": f.path,
                "hash": f.hash,
                "size": f.size
            })).collect::<Vec<_>>(),
            "entrypoints": entrypoints.iter().map(|e| serde_json::json!({
                "symbol": e.symbol,
                "path": e.path
            })).collect::<Vec<_>>(),
            "runtime_dependencies": dependencies.iter().map(|d| serde_json::json!({
                "id": d.id,
                "optional": d.optional
            })).collect::<Vec<_>>()
        },
        "compiled_forms": []
    });

    Ok(source_cadi)
}

fn create_manifest(name: &str, chunk_id: &str) -> Result<serde_json::Value> {
    let manifest_id = format!("app:uuid:{}", uuid::Uuid::new_v4());
    
    let manifest = serde_json::json!({
        "manifest_id": manifest_id,
        "manifest_version": "1.0",
        "application": {
            "name": name,
            "description": format!("{} application", name),
            "version": "0.1.0"
        },
        "build_graph": {
            "nodes": [{
                "id": "main",
                "source_cadi": chunk_id,
                "ir_cadi": null,
                "blob_cadi": null,
                "container_cadi": null,
                "representations": [{
                    "form": "source",
                    "chunk": chunk_id
                }],
                "selection_strategy": "prefer_source"
            }],
            "edges": []
        },
        "build_targets": [{
            "name": "dev",
            "platform": "any",
            "nodes": [{
                "id": "main",
                "prefer": ["source"]
            }]
        }]
    });

    Ok(manifest)
}
