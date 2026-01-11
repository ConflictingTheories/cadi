//! CADI Core - Core types and utilities for Content-Addressed Development Interface
//!
//! This crate provides the fundamental types and utilities used across the CADI ecosystem.

pub mod chunk;
pub mod manifest;
pub mod hash;
pub mod error;

pub use chunk::*;
pub use manifest::*;
pub use hash::*;
pub use error::*;
