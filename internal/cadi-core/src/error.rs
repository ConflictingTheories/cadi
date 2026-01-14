//! Error types for CADI

use thiserror::Error;

/// Main error type for CADI operations
#[derive(Error, Debug)]
pub enum CadiError {
    #[error("Chunk not found: {0}")]
    ChunkNotFound(String),

    #[error("Manifest not found: {0}")]
    ManifestNotFound(String),

    #[error("Invalid chunk ID: {0}")]
    InvalidChunkId(String),

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch {
        expected: String,
        actual: String,
    },

    #[error("Build failed: {0}")]
    BuildFailed(String),

    #[error("Transformation failed: {0}")]
    TransformFailed(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Signature invalid: {0}")]
    SignatureInvalid(String),

    #[error("Trust policy violation: {0}")]
    TrustPolicyViolation(String),

    #[error("Dependency resolution failed: {0}")]
    DependencyResolution(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Graph query error: {0}")]
    GraphQueryError(String),

    #[error("Atomizer error: {0}")]
    AtomizerError(String),

    #[error("Rehydration error: {0}")]
    RehydrationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<serde_json::Error> for CadiError {
    fn from(e: serde_json::Error) -> Self {
        CadiError::Serialization(e.to_string())
    }
}

#[cfg(feature = "ast-parsing")]
impl From<tree_sitter::LanguageError> for CadiError {
    fn from(e: tree_sitter::LanguageError) -> Self {
        CadiError::AtomizerError(e.to_string())
    }
}

#[cfg(feature = "ast-parsing")]
impl From<tree_sitter::QueryError> for CadiError {
    fn from(e: tree_sitter::QueryError) -> Self {
        CadiError::AtomizerError(e.to_string())
    }
}

/// Result type for CADI operations
pub type CadiResult<T> = Result<T, CadiError>;
