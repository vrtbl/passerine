use std::convert::TryFrom;

use crate::common::{
    span::Spanned,
    data::Data,
};

use crate::compiler::ast::ASTPattern;

// TODO: create a pattern specific to the CST?
// Once where (i.e. `x | x > 0`) is added?

/// A pattern that mirrors the structure of some Data.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Symbol(String),
    Data(Data),
    Label(String, Box<Spanned<Pattern>>),
    Tuple(Vec<Spanned<Pattern>>),
    // Where {
    //     pattern: Box<ASTPattern>,
    //     expression: Box<AST>,
    // },
}

impl TryFrom<ASTPattern> for Pattern {
    type Error = String;

    /// Directly maps `ASTPattern`s to `Pattern`s.
    /// This function may become a bit more complex once 'where' is added.
    fn try_from(ast_pattern: ASTPattern) -> Result<Self, Self::Error> {
        Ok(
            match ast_pattern {
                ASTPattern::Symbol(s)   => Pattern::Symbol(s),
                ASTPattern::Data(d)     => Pattern::Data(d),
                ASTPattern::Label(k, a) => Pattern::Label(k, Box::new(a.map(Pattern::try_from)?)),
                ASTPattern::Tuple(t)    => Pattern::Tuple(t.into_iter().map(|i| i.map(Pattern::try_from)).collect::<Result<Vec<_>, _>>()?),
                ASTPattern::Chain(_)    => Err("Unexpected chained construct inside pattern")?,
            }
        )
    }
}

// NOTE: there are a lot of similar items (i.e. binops, (p & e), etc.)
// Store class of item in CST, then delegate exact type to external enum?

/// Represents an item in a desugared`CST`.
/// Each langauge-level construct has it's own `CST` variant.
/// Note that, for instance, call only takes two arguments,
/// Whereas it's originally parsed as a `AST::Form`.
#[derive(Debug, Clone, PartialEq)]
pub enum CST {
    Symbol(String),
    Data(Data),
    Block(Vec<Spanned<CST>>),
    Assign {
        pattern:    Box<Spanned<Pattern>>,
        expression: Box<Spanned<CST>>,
    },
    Lambda {
        pattern:    Box<Spanned<Pattern>>,
        expression: Box<Spanned<CST>>,
    },
    Call {
        fun: Box<Spanned<CST>>,
        arg: Box<Spanned<CST>>,
    },
    Print(Box<Spanned<CST>>),
    Label(String, Box<Spanned<CST>>),
    Tuple(Vec<Spanned<CST>>),
    FFI {
        name:       String,
        expression: Box<Spanned<CST>>,
    },
}

impl CST {
    /// Shortcut for creating an `CST::Assign` variant.
    pub fn assign(
        pattern:    Spanned<Pattern>,
        expression: Spanned<CST>
    ) -> CST {
        CST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating an `CST::Lambda` variant.
    pub fn lambda(
        pattern:    Spanned<Pattern>,
        expression: Spanned<CST>
    ) -> CST {
        CST::Lambda {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating a `CST::Lambda` variant.
    pub fn call(fun: Spanned<CST>, arg: Spanned<CST>) -> CST {
        CST::Call {
            fun: Box::new(fun),
            arg: Box::new(arg),
        }
    }

    // Shortcut for creating an `CST::FFI` variant.
    pub fn ffi(name: &str, expression: Spanned<CST>) -> CST {
        CST::FFI {
            name: name.to_string(),
            expression: Box::new(expression),
        }
    }
}
