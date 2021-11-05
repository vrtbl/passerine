use std::{
    mem,
    collections::HashMap,
};

use crate::common::{
    span::{Span, Spanned},
    lit::Lit,
};

use crate::compiler::syntax::Syntax;

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
    pub fn parse(tokens: Tokens) -> Result<Spanned<AST>, Syntax> {
        // build base parser
        // parse
        // wrap it in a module
        // return it
    }

    /// Looks at the current token and parses a prefix expression, like keywords.
    /// This function will strip preceeding separator tokens.
    fn rule_prefix(&mut self) -> Result<Spanned<AST>, Syntax> {
        let Spanned { item, span } = self.next_token();
        match item {
            Token::Group { delim, .. } => match delim {
                Delim::Curly => self.block(),
                Delim::Paren => self.group(),
                Delim::Square => Syntax::error("Lists are not yet implemented", &span)
            },
            Token::Iden(ref name) => match ResIden::try_new(&name) {
                Some(_) => Err(Syntax::error("This feature is a work in progress", &span)),
                None    => self.symbol(),
            },
            Token::Label(_) => self.label(),
            Token::Data(_)  => self.literal(),
            _               => Err(Syntax::error("Expected an expression", &span)),
        }
    }

    /// Looks at the current token and parses an infix expression like an operator.
    /// Because an operator can be used to split an expression across multiple lines,
    /// this function ignores separator tokens around the operator.
    pub fn rule_infix(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let Spanned { item, span } = self.next_token();
        match item {
            Token::Op(ref name) => match ResOp::try_new(&name)
                .ok_or_else(|_| Syntax::error(
                    &format!("Invalid operator `{}`", name)
                    &span
                ))?
            {
                // TODO: copy over
                // TODO: new vm architecture with effects, get rid of `magic` garbage ffi
                _ => todo!(),
            }
            _ => todo!(),
        }

        todo!()
    }

    /// Parses an expression within a given precedence,
    /// which produces an AST node.
    /// If the expression is empty, returns an empty AST block.
    fn expr(&mut self, prec: Prec, skip_sep: bool) -> Result<Spanned<AST>, Syntax> {
        let mut left = self.rule_prefix()?;

        while !self.is_done() {
            if skip_sep { self.skip_sep() }
            let p = self.prec();
            if self.prec() < prec { break; }
            left = self.rule_infix(left)?;
        }

        Ok(left)
    }
}
