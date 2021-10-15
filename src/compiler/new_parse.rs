use std::collections::HashMap;

use crate::common::{
    span::{Span, Spanned},
    data::Data,
};

use crate::compiler::{lower::Lower, syntax::Syntax};

use crate::construct::{
    token::{Token, Tokens, Delim, ResOp, ResIden},
    tree::{AST, Base, Sugar, Lambda, Pattern, ArgPattern},
    symbol::SharedSymbol,
    module::{ThinModule, Module},
};

/// We're using a Pratt parser, so this little enum
/// defines different precedence levels.
/// Each successive level is higher, so, for example,
/// `* > +`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    /// No precedence.
    None = 0,
    /// `=`
    Assign,
    /// `,`
    Pair,
    /// `:`
    Is,
    /// `->`
    Lambda,
    /// Boolean logic.
    Logic,
    /// `+`, `-`
    AddSub,
    /// `*`, `/`, etc.
    MulDiv,
    /// `**`
    Pow,
    /// `|>`
    Compose,
    /// Implicit function call operator.
    Call,
    /// Highest precedence.
    End,
}

impl Prec {
    /// Increments precedence level to cause the
    /// parser to associate infix operators to the left.
    /// For example, addition is left-associated:
    /// ```build
    /// Prec::Addition.left()
    /// ```
    /// `a + b + c` left-associated becomes `(a + b) + c`.
    /// By default, the parser accociates right.
    ///
    /// Panics if you try to associate left on `Prec::End`,
    /// as this is the highest precedence.
    pub fn left(&self) -> Prec {
        if let Prec::End = self { panic!("Can not associate further left") }
        return unsafe { mem::transmute(self.clone() as u8 + 1) };
    }
}

pub struct Parser {

}

impl Parser {
    pub fn parse(tokens: Tokens) -> Spanned<AST> {
        // build base parser
        // parse
        // wrap it in a module
        // return it
    }

    pub fn expr(&mut self, )
}
