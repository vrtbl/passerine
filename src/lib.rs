//! # Passerine
//! This repository contains the core of the Passerine Programming Language,
//! including the compiler, VM, and various utilities.
//! If you're looking for the documentation for Passerine's CLI, Aspen,
//! you're not in the right place.
//!
//! ## Embedding Passerine in Rust
//! Add passerine to your `Cargo.toml`:
//! ```
//! # make sure it's the latest version
//! passerine = 0.6
//! ```
//! Then simply:
//! ```
//! use passerine;
//!
//! fn main() {
//!     passerine::run("print \"Hello from Passerine!\"");
//! }
//! ```
//! > NOTE: print statements are not yet implemented
//! > They'll be implemented by version 0.11, once the FFI is solidified
//!
//! ## Overview of the compilation process
//! Within the compiler pipeline, source code is represented as a `Source` object
//!


pub mod common;
pub mod compiler;
pub mod vm;
