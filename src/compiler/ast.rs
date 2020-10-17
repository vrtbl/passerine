use crate::common::{
    span::Spanned,
    data::Data,
};

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
    Symbol,
    Data(Data),
    Block(Vec<Spanned<AST>>),
    Pattern(Vec<Spanned<AST>>),
    Form(Vec<Spanned<AST>>),
    Assign {
        pattern:    Box<Spanned<AST>>, // Note - should be pattern
        expression: Box<Spanned<AST>>,
    },
    Lambda {
        pattern:    Box<Spanned<AST>>,
        expression: Box<Spanned<AST>>,
    },
    Print(Box<Spanned<AST>>),
    // TODO: support following constructs as they are implemented
    // Macro {
    //     pattern:    Box<AST>,
    //     expression: Box<AST>,
    // }
}

impl AST {
    /// Shortcut for creating an `AST::Assign` variant.
    pub fn assign(
        pattern:    Spanned<AST>,
        expression: Spanned<AST>
    ) -> AST {
        AST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating an `AST::Lambda` variant.
    pub fn lambda(
        pattern:    Spanned<AST>,
        expression: Spanned<AST>
    ) -> AST {
        AST::Lambda {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }
}
