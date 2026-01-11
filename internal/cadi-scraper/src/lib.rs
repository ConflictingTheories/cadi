//! CADI Scraper/Chunker Utility
//!
//! This module provides utilities for converting source code repositories and file data
//! into reusable CADI chunks. It handles fetching, parsing, semantic chunking, metadata
//! extraction, and publishing to registry servers.

pub mod chunker;
pub mod config;
pub mod error;
pub mod fetcher;
pub mod metadata;
pub mod parser;
pub mod scraper;
pub mod transformer;
pub mod types;

pub use error::{Error, Result};
pub use scraper::Scraper;
pub use types::{ScraperConfig, ScraperInput, ScraperOutput, ChunkingStrategy};
