//! Contains datastructures and utility functions
//! common to both the `compiler` and `vm`.
//!
//! - Core data-strucutres.
//! - Opcodes and number splicing.
//! - Source code representation and span annotations.

pub mod closure;
pub mod data;
pub mod lambda;
pub mod number;
pub mod opcode;
pub mod source;
pub mod span;
pub mod stamp;
