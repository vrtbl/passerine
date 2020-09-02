//! # Passerine
//! This repository contains the core of the Passerine Programming Language,
//! including the compiler, VM, and various utilities.
//! If you're looking for the documentation for Passerine's CLI, Aspen,
//! you're not in the right place.
//!
//! ## Embedding Passerine in Rust
//! > TODO: Clean up crate visibility, create `run` function.
//!
//! Add passerine to your `Cargo.toml`:
//! ```toml
//! # make sure it's the latest version
//! passerine = 0.7
//! ```
//! Then simply:
//! ```ignore
//! // DISCLAIMER: The `run` function used here has not been implemented yet,
//! //             although the underlying interface is mostly stable.
//!
//! use passerine;
//!
//! fn main() {
//!     passerine::run("print \"Hello from Passerine!\"");
//! }
//! ```
//! > NOTE: print statements are not yet implemented.
//! > They'll be implemented by version 0.11, once the FFI is solidified
//!
//! ## Overview of the compilation process
//! Within the compiler pipeline, source code is represented as a `Source` object
//!
//! > TODO: Finish overview.

pub mod common;
pub mod compiler;
pub mod vm;
