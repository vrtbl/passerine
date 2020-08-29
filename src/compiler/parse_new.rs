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
    println!("started parsing");
    let ast = parser.expression(Prec::None)?;
    println!("done");
    parser.consume(Token::End)?;
    return Ok(ast);
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    None,
    Assign,
    Lambda,
    End,
}

impl Prec {
    pub fn escalate(&self) -> Prec {
        if let Prec::End = self { panic!("Can not escalate prec further") }
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
    /// Consumes all separator tokens, returning whether there were any
    fn sep(&mut self) -> bool {
        if self.tokens[self.index].item != Token::Sep { false } else {
            while self.tokens[self.index].item == Token::Sep {
                println!("sepping");
                self.advance();
            };
            true
        }
    }

    /// Returns the current non-sep token then advances the parser.
    fn advance(&mut self) -> &Spanned<Token> {
        println!("advancing");
        self.sep();
        self.index += 1;
        &self.tokens[self.index - 1]
    }

    /// Returns the first non-Sep token.
    fn current(&mut self) -> &Spanned<Token> {
        println!("getting current");
        self.sep();
        &self.tokens[self.index]
    }

    // fn previous(&mut self) -> &Spanned<Token> {
    //     &self.tokens[self.index - 1]
    // }

    // Consumes a specific token then advances the parser.
    // Can be used to consume Sep tokens, which are normally skipped.
    fn consume(&mut self, token: Token) -> Result<&Spanned<Token>, Syntax> {
        println!("consuming {}", token);
        self.index += 1;
        let current = &self.tokens[self.index - 1];
        if current.item != token {
            Err(Syntax::error(&format!("Expected {}, found {}", token, current.item), current.span.clone()))
        } else {
            Ok(current)
        }
    }

    // Core Pratt Parser:

    /// Looks at the current operator token and determines the precedence
    pub fn prec(&mut self) -> Result<Prec, Syntax> {
        let prec = match self.current().item {
            Token::Assign => Prec::Assign,
            Token::Lambda => Prec::Lambda,
            Token::End    => Prec::End,
            _ => Prec::None,
            // maybe prec end?
            // _ => return Err(Syntax::error("Expected an operator", self.current().span.clone())),
        };
        Ok(prec)
    }

    /// Looks at the current token and parses an infix expression
    pub fn rule_prefix(&mut self) -> Result<Spanned<AST>, Syntax> {
        match self.current().item {
            Token::OpenBracket => self.block(),
            Token::OpenParen   => self.call(),
            Token::Symbol      => self.symbol(),
              Token::Number(_)
            | Token::String(_)
            | Token::Boolean(_) => self.literal(),
            _ => Err(Syntax::error("Expected an expression", self.current().span.clone())),
        }
    }

    /// Looks at the current token and parses the right side of any infix expressions.
    pub fn rule_infix(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        match self.current().item {
            Token::Assign => self.assign(left),
            Token::Lambda => self.lambda(left),
            _ => return Err(Syntax::error("Expected another expression", self.current().span.clone()))
        }
    }

    pub fn expression(&mut self, prec: Prec) -> Result<Spanned<AST>, Syntax> {
        let mut left = self.rule_prefix()?;

        while prec <= self.prec()? && self.prec()? != Prec::End {
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

    // Infix:

    fn assign(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let symbol = if let AST::Symbol = left.item { left }
            else { return Err(Syntax::error("Expected a symbol", left.span))? };

        self.consume(Token::Assign)?;
        let expression = self.expression(Prec::Assign)?;
        let combined   = Span::combine(&symbol.span, &expression.span);
        Result::Ok(Spanned::new(AST::assign(symbol, expression), combined))
    }

    fn lambda(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        todo!()
    }

    fn block(&mut self) -> Result<Spanned<AST>, Syntax> {
        todo!()
    }

    fn call(&mut self) -> Result<Spanned<AST>, Syntax> {
        self.consume(Token::OpenParen)?;
        let ast = self.expression(Prec::None)?;
        self.consume(Token::CloseParen)?;
        Ok(ast)
    }
}

#[cfg(test)]
mod test {
    use crate::common::source::Source;
    use crate::compiler::lex::lex;
    use super::*;

    #[test]
    pub fn empty() {
        parse(lex(Source::source("")).unwrap()).unwrap();
    }

    #[test]
    pub fn literal() {
        let ast = parse(lex(Source::source("x = 55.0")).unwrap()).unwrap();
        println!("{:#?}", ast);
        panic!();
    }
}
