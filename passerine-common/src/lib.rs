//! Contains datastructures and utility functions
//! common to both the `compiler` and `vm`.
//!
//! - Core data-strucutres.
//! - Opcodes and number splicing.
//! - Source code representation and span annotations.

pub mod closure;
pub mod data;
pub mod effect;
pub mod inject;
pub mod lambda;
pub mod lit;
pub mod module;
pub mod number;
pub mod opcode;
pub mod source;
pub mod span;
pub mod ty;

pub use closure::Closure;
pub use data::Data;
pub use inject::Inject;
pub use module::Module;
pub use source::Source;
pub use span::{Span, Spanned};
