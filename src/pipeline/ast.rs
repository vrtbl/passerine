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
pub enum AST<'s, 'i> {
    Data(Data),
    Symbol(Local),
    Block(Vec<Spanned<'s, AST<'s, 'i>>>),
    Assign {
        pattern:    Box<Spanned<'s, AST<'s, 'i>>>, // Note - should be pattern
        expression: Box<Spanned<'s, AST<'s, 'i>>>,
    },
    Lambda {
        pattern:    Box<Spanned<'s, AST<'s, 'i>>>,
        expression: Box<Spanned<'s, AST<'s, 'i>>>,
    },
    Call {
        fun: Box<Spanned<'s, AST<'s, 'i>>>,
        arg: Box<Spanned<'s, AST<'s, 'i>>>,
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
impl<'s, 'i> AST<'s, 'i> {
    // Leafs; terminals
    pub fn data(data: Data)      -> AST<'s, 'i> { AST::Data(data)     }
    pub fn symbol(symbol: Local) -> AST<'s, 'i> { AST::Symbol(symbol) }

    // Recursive
    pub fn block(exprs: Vec<Spanned<'s, AST<'s, 'i>>>) -> AST<'s, 'i> { AST::Block(exprs) }

    pub fn assign(
        pattern:    Spanned<'s, AST<'s, 'i>>,
        expression: Spanned<'s, AST<'s, 'i>>
    ) -> AST<'s, 'i> {
        AST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    pub fn lambda(
        pattern:    Spanned<'s, AST<'s, 'i>>,
        expression: Spanned<'s, AST<'s, 'i>>
    ) -> AST<'s, 'i> {
        AST::Lambda {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    pub fn call(
        fun: Spanned<'s, AST<'s, 'i>>,
        arg: Spanned<'s, AST<'s, 'i>>
    ) -> AST<'s, 'i> {
        AST::Call {
            fun: Box::new(fun),
            arg: Box::new(arg)
        }
    }
}
