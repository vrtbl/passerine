// Rewrite of the old parser.
// Pratt parser.
// To be finished on the 29th
use std::mem;
use crate::common::span::{Span, Spanned};

use crate::compiler::{
    syntax::Syntax,
    token::Token,
    ast::AST,
};

pub fn parse(tokens: Vec<Spanned<Token>>) -> Result<Spanned<AST>, Syntax> {
    let mut parser = Parser::new(tokens);
    let ast = parser.body(Token::End)?;
    parser.consume(Token::End)?;
    return Ok(Spanned::new(ast, Span::empty()));
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    None = 0,
    Assign,
    Lambda,
    Call,
    End,
}

impl Prec {
    pub fn associate_left(&self) -> Prec {
        if let Prec::End = self { panic!("Can not associate further left") }
        return unsafe { mem::transmute(self.clone() as u8 + 1) };
    }
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    index:  usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned<Token>>) -> Parser {
        Parser { tokens, index: 0 }
    }

    // Cookie Monster's Helper Functions:

    // NOTE: Maybe don't return bool?
    /// Consumes all seperator tokens, returning whether there were any
    fn sep(&mut self) -> bool {
        if self.tokens[self.index].item != Token::Sep { false } else {
            while self.tokens[self.index].item == Token::Sep {
                self.index += 1;
            };
            true
        }
    }

    /// Returns the current token then advances the parser.
    fn advance(&mut self) -> &Spanned<Token> {
        self.index += 1;
        &self.tokens[self.index - 1]
    }

    /// Returns the first token.
    fn current(&self) -> &Spanned<Token> {
        &self.tokens[self.index]
    }

    /// Returns the first non-Sep token.
    fn skip(&mut self) -> &Spanned<Token> {
        self.sep();
        self.current()
    }

    fn unexpected(&self) -> Syntax {
        let token = self.current();
        Syntax::error(
            &format!("WHAT!? What's {} even doing here? lmao, unexpected", token.item),
            token.span.clone()
        )
    }

    // Consumes a specific token then advances the parser.
    // Can be used to consume Sep tokens, which are normally skipped.
    fn consume(&mut self, token: Token) -> Result<&Spanned<Token>, Syntax> {
        self.index += 1;
        let current = &self.tokens[self.index - 1];
        if current.item != token {
            self.index -= 1;
            Err(Syntax::error(&format!("Expected {}, found {}", token, current.item), current.span.clone()))
        } else {
            Ok(current)
        }
    }

    // Core Pratt Parser:x

    /// Looks at the current token and parses an infix expression
    pub fn rule_prefix(&mut self) -> Result<Spanned<AST>, Syntax> {
        match self.current().item {
            Token::End         => Ok(Spanned::new(AST::Block(vec![]), Span::empty())),

            Token::OpenParen   => self.group(),
            Token::OpenBracket => self.block(),
            Token::Symbol      => self.symbol(),
              Token::Number(_)
            | Token::String(_)
            | Token::Boolean(_) => self.literal(),

            Token::Sep => Err(self.unexpected()),
             _ => Err(Syntax::error("Expected an expression", self.current().span.clone())),
        }
    }

    /// Looks at the current token and parses the right side of any infix expressions.
    pub fn rule_infix(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        match self.current().item {
            Token::Assign => self.assign(left),
            Token::Lambda => self.lambda(left),

            Token::End => Err(self.unexpected()),
            Token::Sep => Err(self.unexpected()),

            _ => self.call(left),
        }
    }

    /// Looks at the current operator token and determines the precedence
    pub fn prec(&mut self) -> Result<Prec, Syntax> {
        let prec = match self.current().item {
            Token::Assign => Prec::Assign,
            Token::Lambda => Prec::Lambda,

              Token::End
            | Token::CloseParen
            | Token::CloseBracket => Prec::End,

            // prefix rules
              Token::OpenParen
            | Token::OpenBracket
            | Token::Symbol
            | Token::Number(_)
            | Token::String(_)
            | Token::Boolean(_) => Prec::Call,

            Token::Sep => Prec::End,
        };
        Ok(prec)
    }

    pub fn expression(&mut self, prec: Prec) -> Result<Spanned<AST>, Syntax> {
        let mut left = self.rule_prefix()?;

        while self.prec()? >= prec
           && self.prec()? != Prec::End
        {
            left = self.rule_infix(left)?;
        }

        return Ok(left);
    }

    // Rule Definitions:

    // Prefix:

    fn symbol(&mut self) -> Result<Spanned<AST>, Syntax> {
        let symbol = self.consume(Token::Symbol)?;
        Ok(Spanned::new(AST::Symbol, symbol.span.clone()))
    }

    fn literal(&mut self) -> Result<Spanned<AST>, Syntax> {
        let Spanned { item: token, span } = self.advance();

        let leaf = match token {
            Token::Number(n)  => AST::Data(n.clone()),
            Token::String(s)  => AST::Data(s.clone()),
            Token::Boolean(b) => AST::Data(b.clone()),
            unexpected => return Err(Syntax::error(
                &format!("Expected a literal, found {:?}", unexpected),
                span.clone()
            )),
        };

        Result::Ok(Spanned::new(leaf, span.clone()))
    }

    fn group(&mut self) -> Result<Spanned<AST>, Syntax> {
        self.consume(Token::OpenParen)?;
        let ast = self.expression(Prec::None.associate_left())?;
        self.consume(Token::CloseParen)?;
        Ok(ast)
    }

    fn body(&mut self, end: Token) -> Result<AST, Syntax> {
        let mut expressions = vec![];

        while self.skip().item != end {
            let ast = self.expression(Prec::None)?;
            expressions.push(ast);
            if let Err(e) = self.consume(Token::Sep) {
                break;
            }
        }

        return Ok(AST::Block(expressions));
    }

    fn block(&mut self) -> Result<Spanned<AST>, Syntax> {
        let start = self.consume(Token::OpenBracket)?.span.clone();
        let ast = self.body(Token::CloseBracket)?;
        let end = self.consume(Token::CloseBracket)?.span.clone();
        return Ok(Spanned::new(ast, Span::combine(&start, &end)));
    }

    // Infix:

    // TODO: assign and lambda are similar... combine?

    fn assign(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let symbol = if let AST::Symbol = left.item { left }
            else { return Err(Syntax::error("Expected a symbol", left.span))? };

        self.consume(Token::Assign)?;
        let expression = self.expression(Prec::Assign)?;
        let combined   = Span::combine(&symbol.span, &expression.span);
        Result::Ok(Spanned::new(AST::assign(symbol, expression), combined))
    }

    fn lambda(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let symbol = if let AST::Symbol = left.item { left }
            else { return Err(Syntax::error("Expected a symbol", left.span))? };

        self.consume(Token::Lambda)?;
        let expression = self.expression(Prec::Lambda)?;
        let combined   = Span::combine(&symbol.span, &expression.span);
        Result::Ok(Spanned::new(AST::lambda(symbol, expression), combined))
    }

    fn call(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let argument = self.expression(Prec::Call.associate_left())?;
        let combined = Span::combine(&left.span, &argument.span);
        return Ok(Spanned::new(AST::call(left, argument), combined));
    }
}

#[cfg(test)]
mod test {
    use crate::common::{
        data::Data,
        source::Source
    };

    use crate::compiler::lex::lex;
    use super::*;

    #[test]
    pub fn empty() {
        let source = Source::source("");
        let ast = parse(lex(source.clone()).unwrap()).unwrap();
        assert_eq!(ast, Spanned::new(AST::Block(vec![]), Span::empty()));
    }

    #[test]
    pub fn literal() {
        let source = Source::source("x = 55.0");
        let ast = parse(lex(source.clone()).unwrap()).unwrap();
        assert_eq!(
            ast,
            Spanned::new(
                AST::assign(
                    Spanned::new(AST::Symbol, Span::new(&source, 0, 1)),
                    Spanned::new(
                        AST::Data(Data::Real(55.0)),
                        Span::new(&source, 4, 4),
                    ),
                ),
                Span::new(&source, 0, 8),
            )
        );
    }

    #[test]
    pub fn complex() {
        let source = Source::source("x -> y -> x y");
        //"\
        // x = {\n    \
        //     w = y -> z -> {\n         \
        //         y = z 2.0 3.0\n        \
        //         x 1.0\n    \
        //     } 17.0\n\
        // }\n\
        // w = { z x y }\n\
        // ");
        let ast = parse(lex(source.clone()).unwrap()).unwrap();
        println!("{}", source.contents);
        println!("{:#?}", ast);
        panic!();
    }
}
