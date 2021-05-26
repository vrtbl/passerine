//! Contains datastructures and utility functions
//! common to both the `compiler` and `vm`.
//!
//! - Core data-strucutres.
//! - Opcodes and number splicing.
//! - Source code representation and span annotations.

pub mod source;
pub mod span;
pub mod data;
pub mod type_;
pub mod number;
pub mod opcode;
pub mod lambda;
pub mod closure;
