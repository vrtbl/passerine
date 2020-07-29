use crate::utils::span::{Spanned};
use crate::vm::data::Data;
use crate::vm::local::Local;

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
#[derive(Debug, Clone)]
pub enum AST {
    Data(Data),
    Symbol(Local),
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

// TODO: Do annotations and nodes need separate lifetimes?
// anns live past the generator, nodes shouldn't
// Additionally, convert to Spanned<AST>?

// TODO: These are semi-reduntant
impl AST {
    // Leafs; terminals
    pub fn data(data: Data)      -> AST { AST::Data(data)     }
    pub fn symbol(symbol: Local) -> AST { AST::Symbol(symbol) }

    // Recursive
    pub fn block(exprs: Vec<Spanned<AST>>) -> AST { AST::Block(exprs) }

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
