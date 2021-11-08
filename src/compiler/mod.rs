//! This module contains the compiler implementation.
//! Note that these modules are public for documentation visiblility,
//! But should never be used outside of the module by `common` or `vm`.
//!
//! Each step in the compiler pipeline turns one datatype into another.
//! loosely, starting with `Source` (string + path):
//!
//! 1. Tokens:   `lex.rs`
//! 2. Absrtact ST: `parse.rs`
//! 3. Concrete ST: `desugar.rs`
//! 4. Scoped ST:  `hoist.rs`
//! 5. Bytecode: `gen.rs`
//!
//! Note that more steps (e.g. ones applying typechecking operations, optimization passes, etc.)
//! may be implemented in the future.

pub mod desugar;
pub mod gen;
pub mod hoist;
pub mod lex;
pub mod parse;

pub mod ast; // high level pre-macro IR
pub mod cst; // post-macro IR
pub mod rule; // macro transformation
pub mod sst;
pub mod token; // hoisted IR

pub mod syntax;

pub use desugar::desugar;
pub use gen::gen;
pub use hoist::hoist;
pub use lex::lex;
pub use parse::parse;
