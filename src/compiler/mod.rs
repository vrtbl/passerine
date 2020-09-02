//! This module contains the compiler implementation.
//! Note that these modules are public for documentation visiblility,
//! But should never be used outside of the module by `common` or `vm`.
//!
//! Each step in the compiler pipeline turns one datatype into another.
//! loosely, starting with `Source` (string + path):
//!
//! 1. Tokens:   `lex.rs`
//! 2. AST:      `parse.rs`
//! 3. Bytecode: `gen.rs`
//!
//! Note that more steps (e.g. ones applying macro transformations, optimization passes, etc.)
//! may be implemented in the future.

pub mod lex;
pub mod parse;
pub mod gen;

pub mod token;
pub mod ast;

pub mod syntax;
