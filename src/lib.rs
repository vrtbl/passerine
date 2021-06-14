//! # Passerine
//! This repository contains the core of the Passerine Programming Language,
//! including the compiler, VM, and various utilities.
//!
//! ## Running Passerine
//! Passerine is primarily run through Aspen,
//! Passerine's package manager.
//! Simply install Aspen, then:
//! ```bash
//! $ aspen new first-package
//! $ cd first-package
//! $ aspen run
//! ```
//! Aspen is the idiomatic way to create and run Passerine packages.
//! The documentation that follows is about the core compiler itself.
//! This is only useful if you're trying to embed Passerine in your Rust project
//! or developing the core Passerine compiler and VM.
//! If that's the case, read on!
//!
//! You can install Passerine by following the instructions at
//! [passerine.io](https://www.passerine.io/#install).
//!
//! ## Embedding Passerine in Rust
//! > TODO: Clean up crate visibility, create `run` function.
//!
//! Add passerine to your `Cargo.toml`:
//! ```toml
//! # make sure this is the latest version
//! passerine = 0.9
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
//!
//! ## Overview of the compilation process
//! > NOTE: For a more detail, read through the documentation
//! for any of the components mentioned.
//!
//! Within the compiler pipeline, source code is represented as a `Source` object.
//! A source is a reference to some code, with an associated path
//! telling which file it came from.
//!
//! Regions of source code can be marked with `Span`s,
//! Which are like `&strs` but with a reference-counted reference to the original `Source`,
//! methods for combining them, and so on.
//! Spans are used throughout the compiler when reporting errors.
//! Compiler Datastructures can be `Spanned` to indicate where they originated.
//!
//! ### Compilation
//! Compilation steps can raise `Err(Syntax)`,
//! indicating that an error occured.
//! `Syntax` is just a `Span` and a message,
//! which can be pretty-printed.
//!
//! The first phase of compilation is lexing.
//! The `Lexer` reads through a source, and produces a stream of `Spanned<Token>`s.
//! The `Lexer` is super simple - it greedily looks for the longest next token,
//! Then consumes it and advances by the token's length.
//! To lex a file, use the `compiler::lex::lex` function.
//!
//! The next phase of compilation is parsing.
//! The parser takes a spanned token stream,
//! and builts a spanned Abstract Syntax Tree (AST).
//! The parser used is a modified Pratt parser.
//! (It's modified to handle the special function-call syntax used.)
//! To parse a token stream, use the `compiler::parse::parse` function.
//!
//! The AST is then traversed and simplified;
//! this is where macro expansion and so on take place.
//! The result is a simplified Concrete Syntax Tree (CST).
//!
//! After constructing the CST, bytecode is generated.
//! Bytecode is just a vector of u8s, interlaced with split numbers.
//! All the opcodes are defined in `common::opcode`,
//! And implemented in `compiler::vm::vm`.
//! A bytecode object is a called a `Lambda`.
//! The bytecode generator works by walking the CST,
//! Recursively nesting itself when a new scope is encountered.
//! To generate bytecode for an CST, use the `compiler::gen::gen` function.
//!
//! ### Execution
//! The VM can raise `Err(Trace)` if it encounters
//! errors during execution.
//! A `Trace` is similar to `Syntax`, but it keeps track of
//! multiple spans representing function calls and so on.
//!
//! After this, raw `Lambda`s are passed to the `VM` to be run.
//! before being run by the `VM`, `Lambdas` are wrapped in `Closure`s,
//! which hold some extra context.
//! To run some bytecode:
//!
//! ```
//! # use passerine::common::{closure::Closure, source::Source};
//! # use passerine::compiler::{lex, parse, desugar, hoist, gen};
//! # use passerine::vm::vm::VM;
//! #
//! # fn main() {
//! # let source = Source::source("pi = 3.14");
//! # let bytecode = Closure::wrap(
//! # lex(source)
//! #     .and_then(parse)
//! #     .and_then(desugar)
//! #     .and_then(hoist)
//! #     .and_then(gen)
//! #     .unwrap());
//! // Initialize a VM with some bytecode:
//! let mut vm = VM::init(bytecode);
//! // Run the initialized VM:
//! vm.run();
//! # }
//! ```
//!
//! The `VM` is just a simple light stack-based VM.

pub mod common;
pub mod core;
pub mod compiler;
// pub mod vm;
pub mod construct;

// exported functions:
// TODO: clean up exports

use std::rc::Rc;
pub use common::{source::Source, closure::Closure, data::Data, span::Spanned};
pub use compiler::{lower::Lower, syntax::Syntax};
pub use construct::module::{ThinModule, Module};
pub use crate::core::ffi::FFI;
// pub use vm::{vm::VM, trace::Trace};

// /// Compiles a [`Source`] to some bytecode.
// pub fn compile(source: Rc<Source>) -> Result<Closure, Syntax> {
//     let tokens   = ThinModule::thin(source).lower()?;
//     let ast      = tokens.lower()?;
//     let cst      = ast.lower()?;
//     let sst      = cst.lower()?;
//     let bytecode = sst.lower()?;
//
//     return Ok(Closure::wrap(bytecode));
// }
//
// /// Compiles a [`Source`] to some bytecode,
// /// With a specific [`FFI`].
// pub fn compile_with_ffi(source: Rc<Source>, ffi: FFI) -> Result<Closure, Syntax> {
//     let tokens   = ThinModule::thin(source).lower()?;
//     let ast      = tokens.lower()?;
//     let cst      = ast.lower()?;
//     let sst      = cst.lower()?;
//     let sst_ffi  = Module::new(sst.repr, (sst.assoc, ffi));
//     let bytecode = sst_ffi.lower()?;
//
//     return Ok(Closure::wrap(bytecode));
// }
//
// /// Run a compiled [`Closure`].
// pub fn run(closure: Closure) -> Result<(), Trace> {
//     let mut vm = VM::init(closure);
//     vm.run()?;
//     Ok(())
// }
