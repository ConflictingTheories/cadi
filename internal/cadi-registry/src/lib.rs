//! CADI Registry - Client for chunk storage and retrieval
//!
//! This crate provides the registry client for interacting with CADI registries.

pub mod client;
pub mod types;
pub mod federation;
pub mod search;
pub mod db;

pub use client::*;
pub use types::*;
pub use federation::*;
pub use search::*;
// Don't export db types to avoid conflicts
