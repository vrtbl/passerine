use crate::common::{
    span::Spanned,
    data::Data,
};

// NOTE: there are a lot of similar items (i.e. binops, (p & e), etc.)
// Store class of item in CST, then delegate exact type to external enum?

/// Represents an item in a desugared`CST`.
/// Each langauge-level construct has it's own `CST` variant.
/// Note that, for instance, call only takes two arguments,
/// Whereas it's originally parsed as a `AST::Form`.
#[derive(Debug, Clone, PartialEq)]
pub enum CST {
    Symbol,
    Data(Data),
    Block(Vec<Spanned<CST>>),
    Assign {
        pattern:    Box<Spanned<CST>>, // Note - should be pattern
        expression: Box<Spanned<CST>>,
    },
    Lambda {
        pattern:    Box<Spanned<CST>>, // Note - should be pattern
        expression: Box<Spanned<CST>>,
    },
    Call {
        fun: Box<Spanned<CST>>,
        arg: Box<Spanned<CST>>,
    },
    Print(Box<Spanned<CST>>),
    // TODO: support following constructs as they are implemented
    // Macro {
    //     pattern:    Box<CST>,
    //     expression: Box<CST>,
    // }
    // Form(Vec<CST>) // function call -> (fun a1 a2 .. an)
}

impl CST {
    /// Shortcut for creating an `CST::Assign` variant.
    pub fn assign(
        pattern:    Spanned<CST>,
        expression: Spanned<CST>
    ) -> CST {
        CST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating an `CST::Lambda` variant.
    pub fn lambda(
        pattern:    Spanned<CST>,
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
}
