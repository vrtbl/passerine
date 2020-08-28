// Rewrite of the old parser.
// Pratt parser.
// To be finished on the 29th

use crate::common::span::Spanned;
use crate::compiler::{
    syntax::Syntax,
    token::Token,
    ast::AST,
};

pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    index:  usize,
}

impl Parser {
    fn advance(&mut self) -> &Spanned<Token> {
        // TODO: check
        let token = &self.tokens[self.index];
        self.index += 1;
        token
    }

    pub fn expression(&mut self, mut prec: usize) -> Result<Spanned<AST>, Syntax> {
        let mut token = self.advance();
        let mut left  = self.nud(token);

        while prec < token.item.left_bind() {
            token = self.advance();
            left  = self.led(left);
        }

        return left;
    }

    fn assign(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let symbol = if let AST::Symbol = left.item { left }
            else { return Err(Syntax::error("Expected a symbol", left.span)); }

        let expression = self.expression(Token::Symbol.left_bind())
        Result::Ok((Spanned::new(AST::assign(symbol, e), combined), remaining))
    }
}
