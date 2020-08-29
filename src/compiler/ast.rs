use crate::common::{
    span::Spanned,
    data::Data,
};

// NOTE: there are a lot of similar items (i.e. binops, (p & e), etc.)
// Store class of item in AST, then delegate exact type to external enum?

/// Represents an item in an AST.
/// Each language-level construct has it's own AST.
/// note that this has two lifetimes:
/// `'s` represents the lifetime of the span,
/// `'i` represents the lifetime of the AST.
/// Spans live through the whole program just about,
/// Whereas the AST is discarded during the bytecode generation phase.
/// Man, explicit lifetime renaming is annoying,
/// and comes across as a code-smell.
/// If you're reading this and think you know a better way.
/// please, at the least, open an issue describing your more optimal methodology.
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
    }
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
    pub fn assign(
        pattern:    Spanned<AST>,
        expression: Spanned<AST>
    ) -> AST {
        AST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    pub fn lambda(
        pattern:    Spanned<AST>,
        expression: Spanned<AST>
    ) -> AST {
        AST::Lambda {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

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
