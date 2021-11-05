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
/// multiplication is higher than addition: `* > +`.
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
    /// Stack of token streams because tokens can be grouped.
    /// The topmost token stream is the one being parsed.
    tokens_stack: Vec<Tokens>,
    /// Stack of locations in the parsing stream.
    /// The topmost token is the current token being looked at.
    indicies: Vec<usize>,
    /// Symbols with the same name are interned.
    /// We don't do this during lexing so that token-based macros
    /// can work with strings.
    symbols: HashMap<String, SharedSymbol>,
}

impl Parser {
    /// parses some tokens into a syntax tree
    pub fn parse(tokens: Tokens) -> Result<Spanned<AST>, Syntax> {
        // build base parser
        let mut parser = Parser {
            tokens_stack: vec![tokens],
            indicies: vec![0],
            symbols: HashMap::new(),
        };

        // parse
        let ast = parser.bare_module(Prec::End, false)?;
        // wrap it in a module

        // return it
        return Ok(ast);
    }

    /// Gets the stream of tokens currently being parsed.
    fn tokens(&self) -> &Tokens {
        &self.tokens_stack.last().unwrap()
    }

    /// Gets the index of the current token being parsed.
    fn index(&self) -> usize {
        *self.indicies.last().unwrap()
    }

    fn index_mut(&mut self) -> &mut usize {
        &mut self.indicies.last().unwrap()
    }

    /// Peeks the current token, does not advance the parser.
    fn peek_token(&self) -> Option<&Spanned<Token>> {
        self.tokens().get(self.index())
    }

    /// Peeks the current non-sep token,
    /// returning None if none exists (i.e. we hit the end of the file).
    /// Does not advance the parser.
    fn peek_non_sep(&self) -> Option<&Spanned<Token>> {
        for i in 0.. {
            let token = self.tokens().get(self.index() + i)?;
            if token.item != Token::Sep { return Some(token); }
        }
        None
    }

    /// Advances the parser by one token.
    fn advance_token(&mut self) {
        *self.index_mut() = self.index() + 1;
    }

    /// Advances the parser until the first non-sep token
    /// Stops advancing if it runs out of tokens
    fn advance_non_sep(&mut self) {
        for i in 0.. {
            match self.tokens().get(self.index() + i) {
                Some(t) if t.item != Token::Sep => break,
                Some(_) => (),
                None => return,
            }
        }
    }

    fn is_done(&self) -> bool {
        self.index() >= self.tokens().len()
    }

    /// Finds the corresponding [`ResOp`] for a string.
    /// Raises a syntax error if the operator string is invalid.
    fn to_op(name: &str, span: Span) -> Result<ResOp, Syntax> {
        ResOp::try_new(&name)
            .ok_or_else(|| Syntax::error(
                &format!("Invalid operator `{}`", name),
                &span,
            ))
    }

    fn to_token<'a>(&self, option: Option<&'a Spanned<Token>>) -> Result<&'a Spanned<Token>, Syntax> {
        option.ok_or_else(|| {
            // TODO: this span is the last token, but it should be just *past* the last token.
            let last_span = self.tokens()
                .last()
                .expect("Can't figure out which file is causing this error")
                .span.clone();

            Syntax::error (
                "Unexpected end of source while parsing",
                &last_span,
            )
        })
    }

    fn prec(&mut self) -> Result<Prec, Syntax> {
        let token = self.to_token(self.peek_token())?;
        let result = match token.item {
            // Delimiters
            | Token::Delim(_, _)
            | Token::Label(_)
            | Token::Iden(_)
            | Token::Lit(_) => Prec::Call,

            Token::Op(name) => match Parser::to_op(&name, token.span)? {
                ResOp::Assign  => Prec::Assign,
                ResOp::Lambda  => Prec::Lambda,
                ResOp::Compose => Prec::Compose,
                ResOp::Pair    => Prec::Pair,

                | ResOp::Add
                | ResOp::Sub => Prec::AddSub,

                | ResOp::Mul
                | ResOp::Div
                | ResOp::Rem => Prec::MulDiv,

                ResOp::Equal => Prec::Logic,
                ResOp::Pow   => Prec::Pow,
            },

            // Unreachable because we skip all all non-sep tokens
            Token::Sep => unreachable!(),
        };

        todo!()
    }

    /// Looks at the current token and parses a prefix expression, like keywords.
    /// This function will strip preceeding separator tokens.
    fn rule_prefix(&mut self) -> Result<Spanned<AST>, Syntax> {
        let token = self.to_token(self.peek_token())?;
        match token.item {
            Token::Delim(delim, _) => match delim {
                Delim::Curly => self.block(),
                Delim::Paren => self.group(),
                Delim::Square => Syntax::error("Lists are not yet implemented", &token.span)
            },
            Token::Iden(ref name) => match ResIden::try_new(&name) {
                Some(_) => Err(Syntax::error("This feature is a work in progress", &token.span)),
                None    => self.symbol(),
            },
            Token::Label(_) => self.label(),
            Token::Lit(_)  => self.literal(),
            _               => Err(Syntax::error("Expected an expression", &token.span)),
        }
    }

    /// Looks at the current token and parses an infix expression like an operator.
    /// Because an operator can be used to split an expression across multiple lines,
    /// this function ignores separator tokens around the operator.
    pub fn rule_infix(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let token = self.to_token(self.peek_token())?;
        let result = match token.item {
            Token::Op(name) => match Parser::to_op(&name, token.span)? {
                ResOp::Assign  => self.assign(left),
                ResOp::Lambda  => self.lambda(left),
                ResOp::Compose => self.compose(left),
                ResOp::Pair    => self.pair(left),
                ResOp::Add     => self.binop(Prec::AddSub.left(), left),
                ResOp::Sub     => self.binop(Prec::AddSub.left(), left),
                ResOp::Mul     => self.binop(Prec::MulDiv.left(), left),
                ResOp::Div     => self.binop(Prec::MulDiv.left(), left),
                ResOp::Rem     => self.binop(Prec::MulDiv.left(), left),
                ResOp::Equal   => self.binop(Prec::Logic.left(),  left),
                ResOp::Pow     => self.binop(Prec::Pow,           left),
            },
            _ => todo!(),
        };

        Ok(result)
    }

    /// Parses an expression within a given precedence,
    /// which produces an AST node.
    /// If the expression is empty, returns an empty AST block.
    fn expr(&mut self, prec: Prec, is_form: bool) -> Result<Spanned<AST>, Syntax> {
        let mut left = self.rule_prefix()?;

        while !self.is_done() {
            if is_form { self.advance_non_sep() }
            let p = self.prec();
            if self.prec()? < prec { break; }
            left = self.rule_infix(left)?;
        }

        Ok(left)
    }
}
