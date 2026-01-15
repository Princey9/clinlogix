//! Library crate for ClinLogix.
//!
//! This exposes programmatic entry points without changing CLI behavior.

pub mod library;
pub mod scan;
pub mod validate;

pub use library::*;
