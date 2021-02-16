use crate::common::{
    span::Spanned,
    data::Data,
};

use crate::compiler::cst::Pattern;

/// Represents an item in a hoisted `SST`.
/// Each langauge-level construct has it's own `SST` variant.
/// Note that symbols have been substituted.
/// At this point in compilation the scope of each local is known.
#[derive(Debug, Clone, PartialEq)]
pub enum SST {
    Symbol(usize),
    Data(Data),
    Block(Vec<Spanned<SST>>),
    Assign {
        pattern:    Box<Spanned<Pattern>>,
        expression: Box<Spanned<SST>>,
    },
    Lambda {
        pattern:    Box<Spanned<Pattern>>,
        expression: Box<Spanned<SST>>,
    },
    Call {
        fun: Box<Spanned<SST>>,
        arg: Box<Spanned<SST>>,
    },
    Print(Box<Spanned<SST>>),
    Label(String, Box<Spanned<SST>>),
    Tuple(Vec<Spanned<SST>>),
    FFI {
        name:       String,
        expression: Box<Spanned<SST>>,
    },
}

pub struct ScopeContext {
    interns: Vec<String>,
}

// impl CST {
//     /// Shortcut for creating an `CST::Assign` variant.
//     pub fn assign(
//         pattern:    Spanned<CSTPattern>,
//         expression: Spanned<CST>
//     ) -> CST {
//         CST::Assign {
//             pattern:    Box::new(pattern),
//             expression: Box::new(expression)
//         }
//     }
//
//     /// Shortcut for creating an `CST::Lambda` variant.
//     pub fn lambda(
//         pattern:    Spanned<CSTPattern>,
//         expression: Spanned<CST>
//     ) -> CST {
//         CST::Lambda {
//             pattern:    Box::new(pattern),
//             expression: Box::new(expression)
//         }
//     }
//
//     /// Shortcut for creating a `CST::Lambda` variant.
//     pub fn call(fun: Spanned<CST>, arg: Spanned<CST>) -> CST {
//         CST::Call {
//             fun: Box::new(fun),
//             arg: Box::new(arg),
//         }
//     }
//
//     // Shortcut for creating an `CST::FFI` variant.
//     pub fn ffi(name: &str, expression: Spanned<CST>) -> CST {
//         CST::FFI {
//             name: name.to_string(),
//             expression: Box::new(expression),
//         }
//     }
// }
