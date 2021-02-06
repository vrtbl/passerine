use std::convert::TryFrom;

use crate::common::{
    span::Spanned,
    data::Data,
};

use crate::compiler::ast::ASTPattern;

// TODO: create a pattern specific to the CST?
// Once where (i.e. `x | x > 0`) is added?

#[derive(Debug, Clone, PartialEq)]
pub enum CSTPattern {
    Symbol(String),
    Data(Data),
    Label(String, Box<Spanned<CSTPattern>>),
    Tuple(Vec<Spanned<CSTPattern>>),
}

impl TryFrom<ASTPattern> for CSTPattern {
    type Error = String;

    fn try_from(ast_pattern: ASTPattern) -> Result<Self, Self::Error> {
        Ok(
            match ast_pattern {
                ASTPattern::Symbol(s)   => CSTPattern::Symbol(s),
                ASTPattern::Data(d)     => CSTPattern::Data(d),
                ASTPattern::Label(k, a) => CSTPattern::Label(k, Box::new(a.map(CSTPattern::try_from)?)),
                ASTPattern::Tuple(t)    => CSTPattern::Tuple(t.into_iter().map(|i| i.map(CSTPattern::try_from)).collect::<Result<Vec<_>, _>>()?),
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
        pattern:    Box<Spanned<CSTPattern>>,
        expression: Box<Spanned<CST>>,
    },
    Lambda {
        pattern:    Box<Spanned<CSTPattern>>,
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
        pattern:    Spanned<CSTPattern>,
        expression: Spanned<CST>
    ) -> CST {
        CST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating an `CST::Lambda` variant.
    pub fn lambda(
        pattern:    Spanned<CSTPattern>,
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
