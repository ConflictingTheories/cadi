use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Chunk creation error: {0}")]
    ChunkCreation(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Metadata extraction error: {0}")]
    Metadata(String),

    #[error("Transformation error: {0}")]
    Transform(String),

    #[error("Fetching error: {0}")]
    Fetch(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}
