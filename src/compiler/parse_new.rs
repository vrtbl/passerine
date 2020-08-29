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

// const generics when? then it's just a [Spanned<AST>, N] for N-fix operators.
// not that passerine really goes any further than infix
type Prefix = Box<dyn FnMut(&mut Parser) -> Result<Spanned<AST>, Syntax>>;
type Infix  = Box<dyn FnMut(&mut Parser, Spanned<AST>) -> Result<Spanned<AST>, Syntax>>;

pub fn parse(tokens: Vec<Spanned<Token>>) -> Result<Spanned<AST>, Syntax> {
    let parser = Parser::new(tokens);
    parser.expression(Prec::None);
    parser.consume(Token::End);
    todo!();
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    None,
    Assign,
    Lambda,
    End,
}

impl Prec {
    pub fn escalate(&self) -> Prec {
        if let Prec::End = self { panic!("Can not escalate prec further") }
        return unsafe { mem::transmute(*self as u8 + 1) };
    }
}

pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    index:  usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned<Token>>) -> Parser {
        Parser { tokens, index: 0 }
    }

    fn advance(&mut self) -> &Spanned<Token> {
        self.index += 1;
        &self.tokens[self.index - 1]
    }


    fn sep(&mut self) -> bool {
        if self.current().item != Token::Sep { false } else {
            while self.current().item == Token::Sep {
                self.advance();
            };
            true
        }
    }

    fn current(&mut self) -> &Spanned<Token> {
        self.sep();
        &self.tokens[self.index]
    }

    fn consume(&mut self, token: Token) -> Result<&Spanned<Token>, Syntax> {
        let next = self.advance();
        if next.item != token {
            Err(Syntax::error(&format!("Expected {}, found {}", token, next.item), next.span.clone()))
        } else {
            Ok(next)
        }
    }

    // NOTE: seems awfully like some sort of dispatch table...
    // I wonder if there's a better way to do this using types
    pub fn prec(&mut self, token: &Token) -> (
        Prec,           // prec(token).0 -> Precedence
        Option<Prefix>, // prec(token).1 -> prefix rule (hence '.1')
        Option<Infix>,  // prec(token).2 -> infix  rule (hence '.2')
    ) {
        match token {
            // prefix ops

            // infix ops
            Token::Assign => (Prec::Assign, None, Some(Box::new(|t| self.assign(t)))),
            Token::Lambda => (Prec::Lambda, None, Some(Box::new(|s, t| Parser::lambda(s, t)))),

            // everything else
            _ => (Prec::None, None, None),
        }
    }

    pub fn rule_infix()

    pub fn expression(&mut self, prec: Prec) -> Result<Spanned<AST>, Syntax> {
        let mut token = self.advance();
        let mut rule = Parser::prec(&token.item).1
            .ok_or(Syntax::error("Expected an expression", token.span.clone()))?;
        let mut left = rule(self)?;

        while prec <= Parser::prec(&token.item).0 {
            let mut rule = Parser::prec(&token.item).2
                .ok_or(Syntax::error("Expected an operater", token.span))?;
            token = self.advance();
            left = rule(self, left)?;
        }

        return Ok(left);
    }

    fn symbol(&mut self) -> Result<Spanned<AST>, Syntax> {
        let token = self.consume(Token::Symbol)?;
        Ok(Spanned::new(AST::Symbol, token.span.clone()))
    }

    fn assign(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let symbol = if let AST::Symbol = left.item { left }
            else { return Err(Syntax::error("Expected a symbol", left.span))? };

        let expression = self.expression(Prec::Assign)?;
        let combined   = Span::combine(&symbol.span, &expression.span);

        // TODO: something's not right...
        Result::Ok(Spanned::new(AST::assign(symbol, expression), combined))
    }

    fn lambda(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        todo!()
    }

    fn literal(&mut self) -> Result<Spanned<AST>, Syntax> {
        let Spanned { item: token, span } = self.current();

        let leaf = match token {
            Token::Number(n)  => AST::Data(n.clone()),
            Token::String(s)  => AST::Data(s.clone()),
            Token::Boolean(b) => AST::Data(b.clone()),
            _ => return Err(Syntax::error("Unexpected token", span.clone())),
        };

        Result::Ok(Spanned::new(leaf, span.clone()))
    }

    fn call(&mut self) -> Result<Spanned<AST>, Syntax> {
        let ast = self.expression(Prec::None)?;
        self.consume(Token::CloseParen)?;
        Ok(ast)
    }
}
