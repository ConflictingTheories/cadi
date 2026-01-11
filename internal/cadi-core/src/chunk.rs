//! Chunk types and operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of CADI chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CadiType {
    Source,
    Intermediate,
    Blob,
    Container,
}

/// Metadata for a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMeta {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// What a chunk provides
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkProvides {
    #[serde(default)]
    pub concepts: Vec<String>,
    #[serde(default)]
    pub interfaces: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<String>,
}

/// Licensing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkLicensing {
    pub license: String,
    #[serde(default)]
    pub restrictions: Vec<String>,
}

/// Chunk lineage (provenance)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkLineage {
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_receipt: Option<String>,
}

/// Base chunk structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub chunk_id: String,
    pub cadi_type: CadiType,
    pub meta: ChunkMeta,
    #[serde(default)]
    pub provides: ChunkProvides,
    pub licensing: ChunkLicensing,
    #[serde(default)]
    pub lineage: ChunkLineage,
}

/// A file entry in source CADI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,
}

/// An entrypoint in source CADI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entrypoint {
    pub symbol: String,
    pub path: String,
}

/// A dependency declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub features: Vec<String>,
}

/// Source-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceData {
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dialect: Option<String>,
    pub files: Vec<SourceFile>,
    #[serde(default)]
    pub entrypoints: Vec<Entrypoint>,
    #[serde(default)]
    pub runtime_dependencies: Vec<Dependency>,
}

/// Source CADI chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCadi {
    #[serde(flatten)]
    pub chunk: Chunk,
    pub source: SourceData,
    #[serde(default)]
    pub compiled_forms: Vec<CompiledForm>,
}

/// Link to a compiled form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledForm {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir_cadi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_cadi: Option<String>,
    #[serde(default)]
    pub architectures: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deterministic: Option<bool>,
}

/// IR module data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrModule {
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,
    #[serde(default)]
    pub exports: Vec<ModuleExport>,
}

/// Module export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleExport {
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// IR-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateData {
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub module: IrModule,
    #[serde(default)]
    pub imports: Vec<ModuleImport>,
}

/// Module import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleImport {
    pub module: String,
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// IR CADI chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrCadi {
    #[serde(flatten)]
    pub chunk: Chunk,
    pub intermediate: IntermediateData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_link: Option<SourceLink>,
    #[serde(default)]
    pub compiled_forms: Vec<CompiledForm>,
}

/// Link to source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_cadi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,
}

/// Binary blob entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobEntry {
    pub architecture: String,
    pub format: String,
    pub hash: String,
    pub size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linking: Option<LinkingInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_info: Option<BuildInfo>,
}

/// Linking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkingInfo {
    #[serde(rename = "type")]
    pub linking_type: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Security features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    #[serde(default)]
    pub pie: bool,
    #[serde(default)]
    pub nx: bool,
    #[serde(default)]
    pub stack_canary: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relro: Option<String>,
}

/// Build information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reproducible: Option<bool>,
}

/// Blob CADI chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobCadi {
    #[serde(flatten)]
    pub chunk: Chunk,
    pub blobs: Vec<BlobEntry>,
}

/// Container-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerData {
    pub format: String,
    pub image_ref: String,
    #[serde(default)]
    pub layers: Vec<ContainerLayer>,
    #[serde(default)]
    pub entrypoint: Vec<String>,
    #[serde(default)]
    pub cmd: Vec<String>,
    #[serde(default)]
    pub environment: Vec<EnvVar>,
    #[serde(default)]
    pub exposed_ports: Vec<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(default = "default_os")]
    pub os: String,
}

fn default_os() -> String {
    "linux".to_string()
}

/// Container layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerLayer {
    pub hash: String,
    pub size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadi_chunk: Option<String>,
}

/// Environment variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

/// Container CADI chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerCadi {
    #[serde(flatten)]
    pub chunk: Chunk,
    pub container: ContainerData,
    #[serde(default)]
    pub deployment_target: Vec<String>,
}

impl Chunk {
    /// Create a new chunk with the given ID and type
    pub fn new(chunk_id: String, cadi_type: CadiType, name: String) -> Self {
        Self {
            chunk_id,
            cadi_type,
            meta: ChunkMeta {
                name,
                description: None,
                version: None,
                tags: Vec::new(),
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                updated_at: None,
            },
            provides: ChunkProvides::default(),
            licensing: ChunkLicensing {
                license: "MIT".to_string(),
                restrictions: Vec::new(),
            },
            lineage: ChunkLineage::default(),
        }
    }
}
