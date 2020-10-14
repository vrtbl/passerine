//! Contains datastructures and utility functions
//! common to both the `compiler` and `vm`, including:
//!
//! - Core data-strucutres.
//! - Opcodes and number splicing.
//! - Source code representation and span annotations.

pub mod source;
pub mod span;
pub mod data;
pub mod number;
pub mod opcode;
pub mod lambda;
