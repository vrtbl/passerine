//! This module contains the compiler implementation.
//! This module also implements
//!
//! Note that more steps (e.g. ones applying typechecking
//! operations, optimization passes, etc.)
//! may be implemented in the future.

pub mod lex;
pub use lex::Lexer;

pub mod read;
pub use read::Reader;

// pub mod expand;
// pub use expand::Expander;

pub mod parse;
pub use parse::Parser;

// pub mod desugar;
// pub use desugar::Desugarer;

// pub mod hoist;
// pub use hoist::Hoister;

// // pub mod unify;
// pub mod gen;
// pub use gen::Compiler;

pub mod syntax;
pub use syntax::Syntax;

use std::{collections::HashMap, rc::Rc};

use crate::{
    common::{lambda::Lambda, Source, Spanned},
    construct::{
        scope::Scope,
        symbol::SharedSymbol,
        token::{TokenTree, Tokens},
        tree::{AST, CST, SST},
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

// #[inline(always)]
// pub fn desugar(
//     source: Rc<Source>,
// ) -> Result<(Spanned<CST>, HashMap<String, SharedSymbol>), Syntax> {
//     let (ast, symbols) = parse(source)?;
//     Ok((Desugarer::desugar(ast), symbols))
// }

// #[inline(always)]
// pub fn hoist(source: Rc<Source>) -> Result<(Spanned<SST>, Scope), Syntax> {
//     let (cst, symbols) = desugar(source)?;
//     Hoister::hoist(cst, symbols)
// }

// #[inline(always)]
// pub fn gen(source: Rc<Source>) -> Result<Rc<Lambda>, Syntax> {
//     let (sst, scope) = hoist(source)?;
//     Compiler::compile(sst, scope)
// }

// #[inline(always)]
// pub fn compile_sst(
//     sst: Spanned<SST>,
//     scope: Scope,
// ) -> Result<Rc<Lambda>, Syntax> {
//     Compiler::compile(sst, scope)
// }

// // TODO: convert symbols to type alias somewhere
// #[inline(always)]
// pub fn compile_cst(
//     cst: Spanned<CST>,
//     symbols: HashMap<String, SharedSymbol>,
// ) -> Result<Rc<Lambda>, Syntax> {
//     let (sst, scope) = Hoister::hoist(cst, symbols)?;
//     compile_sst(sst, scope)
// }

// #[inline(always)]
// pub fn compile_ast(
//     ast: Spanned<AST>,
//     symbols: HashMap<String, SharedSymbol>,
// ) -> Result<Rc<Lambda>, Syntax> {
//     let cst = Desugarer::desugar(ast);
//     compile_cst(cst, symbols)
// }

// #[inline(always)]
// pub fn compile_token_tree(
//     token_tree: Spanned<TokenTree>,
// ) -> Result<Rc<Lambda>, Syntax> {
//     let (ast, symbols) = Parser::parse(token_tree)?;
//     compile_ast(ast, symbols)
// }

// #[inline(always)]
// pub fn compile_tokens(tokens: Spanned<Tokens>) -> Result<Rc<Lambda>, Syntax> {
//     let token_tree = Reader::read(tokens)?;
//     compile_token_tree(token_tree)
// }

// #[inline(always)]
// pub fn compile_source(source: Rc<Source>) -> Result<Rc<Lambda>, Syntax> {
//     let tokens = Lexer::lex(source)?;
//     compile_tokens(tokens)
// }
