// Each step in the compiler pipeline turns one datatype into another.
// loosely:
// ~> Source (string)
// -> Tokens          : lex.rs
// -> AST             : parse.rs
// -> Bytecode        : gen.rs
// ~> Run (result)    : vm.rs

pub mod lex;
pub mod parse;
pub mod gen;

mod token;
mod ast;

mod syntax;
