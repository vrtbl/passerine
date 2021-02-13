use std::convert::TryFrom;

use crate::common::{
    span::Spanned,
    data::Data,
};

/// Represents a Pattern during the AST phase of compilation.
/// A pattern is like a very general type,
/// because Passerine uses structural row-based typing.
#[derive(Debug, Clone, PartialEq)]
pub enum ASTPattern {
    Symbol(String),
    Data(Data),
    Chain(Vec<Spanned<ASTPattern>>), // used inside lambdas
    Label(String, Box<Spanned<ASTPattern>>),
    Tuple(Vec<Spanned<ASTPattern>>),
    // Where {
    //     pattern: Box<ASTPattern>,
    //     expression: Box<AST>,
    // },
}

impl ASTPattern {
    // Shortcut for creating a `Pattern::Label` variant
    pub fn label(name: String, pattern: Spanned<ASTPattern>) -> ASTPattern {
        ASTPattern::Label(name, Box::new(pattern))
    }
}

impl TryFrom<AST> for ASTPattern {
    type Error = String;

    /// Tries to convert an `AST` into a `Pattern`.
    /// Patterns mirror the `AST`s they are designed to destructure.
    /// During parsing, they are just parsed as `AST`s -
    /// When the compiler can determine that an AST is actually a pattern,
    /// It performs this conversion.
    fn try_from(ast: AST) -> Result<Self, Self::Error> {
        Ok(
            match ast {
                AST::Symbol(s) => ASTPattern::Symbol(s),
                AST::Data(d) => ASTPattern::Data(d),
                AST::Label(k, a) => ASTPattern::Label(k, Box::new(a.map(ASTPattern::try_from)?)),
                AST::Pattern(p) => p,
                AST::Form(f) => {
                    let mut patterns = vec![];
                    for item in f {
                        patterns.push(item.map(ASTPattern::try_from)?);
                    }
                    ASTPattern::Chain(patterns)
                },
                AST::Tuple(t) => {
                    let mut patterns = vec![];
                    for item in t {
                        patterns.push(item.map(ASTPattern::try_from)?);
                    }
                    ASTPattern::Tuple(patterns)
                }
                AST::Group(e) => e.map(ASTPattern::try_from)?.item,
                _ => Err("Unexpected construct inside pattern")?,
            }
        )
    }
}

/// Represents an argument pattern,
/// i.e. the mini language used to match macros.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgPat {
    Keyword(String),
    Symbol(String),
    Group(Vec<Spanned<ArgPat>>),
}

impl TryFrom<AST> for ArgPat {
    type Error = String;

    /// Like `ASTPattern`s, `ArgPat`s are represented as ASTs,
    /// Then converted into `ArgPat`s when the compiler determines it so.
    fn try_from(ast: AST) -> Result<Self, Self::Error> {
        Ok(
            match ast {
                AST::Symbol(s) => ArgPat::Symbol(s),
                AST::ArgPat(p) => p,
                AST::Form(f) => {
                    let mut mapped = vec![];
                    for a in f { mapped.push(a.map(ArgPat::try_from)?); }
                    ArgPat::Group(mapped)
                }
                _ => Err("Unexpected construct inside argument pattern")?,
            }
        )
    }
}

/// Represents an item in a sugared `AST`.
/// Which is the direct result of parsing
/// Each syntax-level construct has it's own `AST` variant.
/// When macros are added, for instance, they will be here,
/// But not in the `CST`, which is the desugared syntax tree,
/// and represents language-level constructs
#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    Symbol(String),
    Data(Data),
    Block(Vec<Spanned<AST>>),
    Form(Vec<Spanned<AST>>),
    Group(Box<Spanned<AST>>),
    Pattern(ASTPattern),
    ArgPat(ArgPat),
    Tuple(Vec<Spanned<AST>>),
    Assign {
        pattern:    Box<Spanned<ASTPattern>>,
        expression: Box<Spanned<AST>>,
    },
    Lambda {
        pattern:    Box<Spanned<ASTPattern>>,
        expression: Box<Spanned<AST>>,
    },
    Composition {
        argument: Box<Spanned<AST>>,
        function: Box<Spanned<AST>>,
    },
    Print(Box<Spanned<AST>>),
    Label(String, Box<Spanned<AST>>),
    Syntax {
        arg_pat:    Box<Spanned<ArgPat>>,
        expression: Box<Spanned<AST>>,
    },
    // TODO: Currently quite basic
    // Use a symbol or the like?
    FFI {
        name:       String,
        expression: Box<Spanned<AST>>,
    },
}

impl AST {
    /// Shortcut for creating an `AST::Assign` variant.
    pub fn assign(
        pattern:    Spanned<ASTPattern>,
        expression: Spanned<AST>
    ) -> AST {
        AST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating an `AST::Lambda` variant.
    pub fn lambda(
        pattern:    Spanned<ASTPattern>,
        expression: Spanned<AST>
    ) -> AST {
        AST::Lambda {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    pub fn composition(
        argument: Spanned<AST>,
        function: Spanned<AST>,
    ) -> AST {
        AST::Composition {
            argument: Box::new(argument),
            function: Box::new(function),
        }
    }

    /// Shortcut for creating an `AST::Syntax` variant.
    /// i.e. a macro definition
    pub fn syntax(
        arg_pat: Spanned<ArgPat>,
        expression: Spanned<AST>,
    ) -> AST {
        AST::Syntax {
            arg_pat:    Box::new(arg_pat),
            expression: Box::new(expression),
        }
    }

    // Shortcut for creating an `AST::FFI` variant.
    pub fn ffi(name: &str, expression: Spanned<AST>) -> AST {
        AST::FFI {
            name: name.to_string(),
            expression: Box::new(expression),
        }
    }

    // Shortcut for creating an `AST::Group` variant.
    pub fn group(expression: Spanned<AST>) -> AST {
        AST::Group(Box::new(expression))
    }
}
