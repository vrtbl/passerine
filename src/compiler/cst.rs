use crate::common::{
    span::Spanned,
    data::Data,
};

// NOTE: there are a lot of similar items (i.e. binops, (p & e), etc.)
// Store class of item in CST, then delegate exact type to external enum?

/// Represents an item in an `CST`.
/// Each language-level construct has it's own `CST` variant.
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
        pattern:    Box<Spanned<CST>>,
        expression: Box<Spanned<CST>>,
    },
    Call {
        fun: Box<Spanned<CST>>,
        arg: Box<Spanned<CST>>,
    },
    Print(Box<Spanned<CST>>),
    // TODO: support following constructs as they are implemented
    // Lambda {
    //     pattern:    Box<CST>, // Note - should be pattern
    //     expression: Box<CST>,
    // },
    // Macro {
    //     pattern:    Box<CST>,
    //     expression: Box<CST>,
    // }
    // Form(Vec<CST>) // function call -> (fun a1 a2 .. an)
}
