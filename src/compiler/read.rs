use crate::common::span::{Span, Spanned};
use crate::compiler::syntax::{Note, Syntax};
use crate::construct::token::{Delim, Token, TokenTree, TokenTrees, Tokens};

pub struct Reader {
    tokens: Tokens,
    index: usize,
    // stack of nested groupings
    opening: Vec<(usize, Spanned<Delim>)>,
}

impl Reader {
    pub fn read(tokens: Tokens) -> Result<TokenTree, Syntax> {
        let mut reader = Reader {
            tokens,
            index: 0,
            opening: vec![],
        };

        reader.block()
    }

    /// Skips over all separator tokens.
    fn skip_sep(&mut self) {
        while self.index < self.tokens.len() {
            if self.tokens[self.index].item != Token::Sep {
                return;
            }
            self.index += 1;
        }
    }

    /// Returns the next token, advancing the lexer by 1.
    fn next_token(&mut self) -> Option<Spanned<Token>> {
        if self.index < self.tokens.len() {
            let token = &self.tokens[self.index];
            self.index += 1;
            // We can clone here because it's not that expensive
            Some(token.clone())
        } else {
            None
        }
    }

    /// Performs a trivial identity conversion.
    /// If the conversion is not trivial, this returns None.
    /// Yes, I know this is opaque.
    /// I plan to refactor this out eventually.
    fn trivial(token: Token) -> Option<TokenTree> {
        let token = match token {
            Token::Iden(iden) => TokenTree::Iden(iden),
            Token::Label(label) => TokenTree::Label(label),
            Token::Op(op) => TokenTree::Op(op),
            Token::Lit(lit) => TokenTree::Lit(lit),
            _ => return None,
        };

        Some(token)
    }

    fn form(&mut self) -> Result<TokenTrees, Syntax> {
        let mut tokens = vec![];

        while let Some(token) = self.next_token() {
            let span = token.span;
            let item = match token.item {
                Token::Open(delim) => {
                    self.enter_group(Spanned::new(delim, span.clone()))?
                },
                Token::Close(delim) => {
                    self.exit_group(Spanned::new(delim, span.clone()))?;
                    break;
                },
                Token::Sep => continue,

                // Trivial conversion
                other => Self::trivial(other).unwrap(),
            };

            let combined = Spanned::new(item, span);
            tokens.push(combined);
        }

        return Ok(tokens);
    }

    fn line_form(&mut self) -> Result<TokenTree, Syntax> {
        // clear out leading separators
        self.skip_sep();

        let mut lines = vec![];
        let mut line = vec![];
        let mut after_op = false;

        while let Some(token) = self.next_token() {
            let span = token.span;
            let item = match token.item {
                Token::Open(delim) => {
                    self.enter_group(Spanned::new(delim, span.clone()))?
                },
                Token::Close(delim) => {
                    self.exit_group(Spanned::new(delim, span.clone()))?;
                    break;
                },
                Token::Sep => {
                    if !after_op {
                        lines.push(line);
                        line = vec![];
                    }
                    continue;
                },

                Token::Op(op) => {
                    TokenTree::Op(op);
                    after_op = true;
                    continue;
                },

                // Trivial conversion
                other => Self::trivial(other).unwrap(),
            };

            after_op = false;
            let combined = Spanned::new(item, span);
            line.push(combined);
        }

        todo!()
    }

    fn block(&mut self) -> Result<TokenTree, Syntax> {
        self.line_form();
        todo!()
    }

    fn enter_group(
        &mut self,
        delim: Spanned<Delim>,
    ) -> Result<TokenTree, Syntax> {
        self.opening.push((self.index, delim.clone()));

        let tree = match delim.item {
            Delim::Curly => self.block()?,
            Delim::Paren => TokenTree::Form(self.form()?),
            Delim::Square => TokenTree::List(self.form()?),
        };

        Ok(tree)
    }

    fn exit_group(
        &mut self,
        closing_delim: Spanned<Delim>,
    ) -> Result<(), Syntax> {
        let (index, opening_delim) = self.opening.pop().ok_or_else(|| {
            Syntax::error(
                &format!("Unexpected closing {}", closing_delim.item),
                &closing_delim.span,
            )
        })?;

        if opening_delim.item == opening_delim.item {
            return Ok(());
        }

        let error = Syntax::error(
            &format!(
                "Mismatched opening {} and closing {}",
                closing_delim.item, opening_delim.item,
            ),
            &Span::combine(&opening_delim.span, &closing_delim.span),
        )
        .add_note(Note::new(opening_delim.span))
        .add_note(Note::new(closing_delim.span));

        Err(error)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn random_tokens(tokens: Token) {
            dbg!(tokens);
        }
    }
}
