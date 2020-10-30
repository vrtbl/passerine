use crate::common::{
    span::Spanned,
    data::Data,
};

// TODO: separate patterns and argument patterns?

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Symbol(String),
    Data(Data),
    Label(String, Box<Spanned<Pattern>>),
    Where {
        pattern: Box<Spanned<Pattern>>,
        conditions: Box<Spanned<AST>>,
    }
}

pub enum ArgPattern {
    Keyword(String),
    Symbol(String),
    
}

// NOTE: there are a lot of similar items (i.e. binops, (p & e), etc.)
// Store class of item in AST, then delegate exact type to external enum?

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
    Pattern(Pattern),
    Assign {
        pattern:    Box<Spanned<Pattern>>,
        expression: Box<Spanned<AST>>,
    },
    Lambda {
        pattern:    Box<Spanned<Pattern>>,
        expression: Box<Spanned<AST>>,
    },
    Print(Box<Spanned<AST>>),
    Label(String, Box<Spanned<AST>>),
    Syntax {
        patterns:   Box<Spanned<Vec<Spanned<Pattern>>>>,
        expression: Box<Spanned<AST>>,
    }
}

impl AST {
    /// Shortcut for creating an `AST::Assign` variant.
    pub fn assign(
        pattern:    Spanned<Pattern>,
        expression: Spanned<AST>
    ) -> AST {
        AST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating an `AST::Lambda` variant.
    pub fn lambda(
        pattern:    Spanned<Pattern>,
        expression: Spanned<AST>
    ) -> AST {
        AST::Lambda {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating an `AST::Syntax` variant.
    /// i.e. a macro definition
    pub fn syntax(
        pattern: Spanned<Vec<Spanned<Pattern>>>,
        expression: Spanned<AST>,
    ) -> AST {
        AST::Syntax {
            patterns:   Box::new(pattern),
            expression: Box::new(expression),
        }
    }
}
