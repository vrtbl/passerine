use crate::common::{
    span::Spanned,
    data::Data,
};

// NOTE: there are a lot of similar items (i.e. binops, (p & e), etc.)
// Store class of item in AST, then delegate exact type to external enum?

/// Represents an item in an `AST`.
/// Each language-level construct has it's own `AST` variant.
#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    Symbol,
    Data(Data),
    Block(Vec<Spanned<AST>>),
    Assign {
        pattern:    Box<Spanned<AST>>, // Note - should be pattern
        expression: Box<Spanned<AST>>,
    },
    Lambda {
        pattern:    Box<Spanned<AST>>,
        expression: Box<Spanned<AST>>,
    },
    Call {
        fun: Box<Spanned<AST>>,
        arg: Box<Spanned<AST>>,
    },
    Print(Box<Spanned<AST>>),
    // TODO: support following constructs as they are implemented
    // Lambda {
    //     pattern:    Box<AST>, // Note - should be pattern
    //     expression: Box<AST>,
    // },
    // Macro {
    //     pattern:    Box<AST>,
    //     expression: Box<AST>,
    // }
    // Form(Vec<AST>) // function call -> (fun a1 a2 .. an)
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

    // TODO: make a call a list of items rather than a left-associated tree?
    /// Shortcut for creating an `AST::Call` variant.
    pub fn call(
        fun: Spanned<AST>,
        arg: Spanned<AST>
    ) -> AST {
        AST::Call {
            fun: Box::new(fun),
            arg: Box::new(arg)
        }
    }
}
