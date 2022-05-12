//! This module contains the compiler implementation.
//! Note that these modules are public for documentation
//! visiblility, But should never be used outside of the
//! module by `common` or `vm`.
//!
//! Each step in the compiler pipeline turns one datatype
//! into another. loosely, starting with `Source` (string +
//! path):
//!
//! 1. Tokens:   `lex.rs`
//! 2. Absrtact ST: `parse.rs`
//! 3. Concrete ST: `desugar.rs`
//! 4. Scoped ST:  `hoist.rs`
//! 5. Bytecode: `gen.rs`
//!
//! Note that more steps (e.g. ones applying typechecking
//! operations, optimization passes, etc.)
//! may be implemented in the future.

pub mod lex;
use std::{
    collections::HashMap,
    rc::Rc,
};

pub use lex::Lexer;

pub mod read;
pub use read::Reader;
// pub mod expand;
pub mod parse;
pub use parse::Parser;

pub mod desugar;
pub use desugar::Desugarer;

pub mod hoist;
pub use hoist::Hoister;
// pub mod unify;
pub mod gen;
pub use gen::Compiler;

pub mod syntax;
pub use syntax::Syntax;

use crate::{
    common::{
        lambda::Lambda,
        Source,
        Spanned,
    },
    construct::{
        scope::Scope,
        symbol::{
            SharedSymbol,
            SymbolTable,
        },
        token::{
            TokenTree,
            Tokens,
        },
        tree::{
            AST,
            CST,
            SST,
        },
    },
};

#[inline(always)]
pub fn lex(source: Rc<Source>) -> Result<Spanned<Tokens>, Syntax> {
    Lexer::lex(source)
}

#[inline(always)]
pub fn read(source: Rc<Source>) -> Result<Spanned<TokenTree>, Syntax> {
    let tokens = lex(source)?;
    Reader::read(tokens)
}

#[inline(always)]
pub fn parse(
    source: Rc<Source>,
) -> Result<(Spanned<AST>, HashMap<String, SharedSymbol>), Syntax> {
    let token_tree = read(source)?;
    Parser::parse(token_tree)
}

#[inline(always)]
pub fn desugar(
    source: Rc<Source>,
) -> Result<(Spanned<CST>, HashMap<String, SharedSymbol>), Syntax> {
    let (ast, symbols) = parse(source)?;
    Ok((Desugarer::desugar(ast), symbols))
}

#[inline(always)]
pub fn hoist(source: Rc<Source>) -> Result<(Spanned<SST>, Scope), Syntax> {
    let (cst, symbols) = desugar(source)?;
    Hoister::hoist(cst, symbols)
}

#[inline(always)]
pub fn gen(source: Rc<Source>) -> Result<Rc<Lambda>, Syntax> {
    let (sst, scope) = hoist(source)?;
    Compiler::compile(sst, scope)
}
