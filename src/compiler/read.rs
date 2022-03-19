use crate::common::span::{Span, Spanned};
use crate::compiler::syntax::{Note, Syntax};
use crate::construct::token::{Delim, Token, TokenTree, TokenTrees, Tokens};

pub struct Reader {
    tokens: Tokens,
    index: usize,
    // stack of nested groupings
    opening: Vec<Spanned<Delim>>,
}

impl Reader {
    pub fn read(tokens: Tokens) -> Result<TokenTree, Syntax> {
        let mut reader = Reader {
            tokens,
            index: 0,
            opening: vec![],
        };

        let result = reader.block();

        // if there are still unclosed delimiters on the opening stack
        if !reader.opening.is_empty() {
            let still_opened = reader.opening.last().unwrap();
            Err(Syntax::error(
                &format!("Unclosed opening {}", still_opened.item,),
                &still_opened.span,
            ))
        } else {
            result
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
            _ => panic!(),
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

    fn block(&mut self) -> Result<TokenTree, Syntax> {
        let mut lines = vec![];
        let mut line = vec![];
        let mut after_sep = false;
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
                    after_sep = true;
                    continue;
                },

                Token::Op(op) => {
                    let spanned = Spanned::new(TokenTree::Op(op), span);
                    line.push(spanned);
                    after_sep = false;
                    after_op = true;
                    continue;
                },

                // Trivial conversion
                other => Self::trivial(other).unwrap(),
            };

            if after_sep && !after_op && !line.is_empty() {
                lines.push(line);
                line = vec![];
            }

            after_sep = false;
            after_op = false;
            let combined = Spanned::new(item, span);
            line.push(combined);
        }

        lines.push(line);
        Ok(TokenTree::Block(lines))
    }

    fn enter_group(
        &mut self,
        delim: Spanned<Delim>,
    ) -> Result<TokenTree, Syntax> {
        self.opening.push(delim.clone());

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
        let opening_delim = self.opening.pop().ok_or_else(|| {
            Syntax::error(
                &format!("Unexpected closing {}", closing_delim.item),
                &closing_delim.span,
            )
        })?;

        if opening_delim.item == closing_delim.item {
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
    use std::fmt::Write;

    use super::*;
    use crate::common::source::Source;
    use crate::compiler::lex::Lexer;

    use proptest::prelude::*;

    /// Generates a source file from some tokens.
    /// Replaces each token with a minimal representative token.
    /// for example, all delimiters become curly brackets,
    /// all identifiers become `x`, and so on.
    fn generate_minimal_source(tokens: &[Token]) -> String {
        let mut buffer = String::new();
        for token in tokens {
            let new = match token {
                Token::Open(_) => "{",
                Token::Close(_) => "}",
                Token::Sep => "\n",
                Token::Iden(_) => " x ",
                Token::Label(_) => " X ",
                Token::Op(_) => " + ",
                Token::Lit(_) => " 2 ",
            };
            buffer.write_str(new).unwrap();
        }
        buffer
    }

    /// Checks if there are a matching number of opening and closing delims.
    /// Doesn't care if delims are of different types
    /// To be used with `generate_minimal_source`.
    fn check_if_balanced(tokens: &[Token]) -> bool {
        let mut delims = 0;

        for token in tokens {
            match token {
                Token::Open(_) => delims += 1,
                Token::Close(_) => delims -= 1,
                _ => continue,
            };

            if delims < 0 {
                return false;
            }
        }

        delims == 0
    }

    proptest! {
        #[test]
        fn check_balance(tokens: Vec<Token>) {
            let balanced = check_if_balanced(&tokens);
            let source = generate_minimal_source(&tokens);
            dbg!(&source);
            let tokens = Lexer::lex(Source::source(&source)).unwrap();
            let token_tree = Reader::read(tokens);

            if balanced {
                println!("balanced");
                assert!(token_tree.is_ok())
            } else {
                println!("unbalanced");
                assert!(token_tree.is_err())
            }
        }

        #[test]
        fn multiline_ops(splits: Vec<bool>) {
            // generate a random valid buffer
            let mut buffer = "2".to_string();
            for (index, split) in splits.iter().enumerate() {
                let sep = if *split { '\n' } else { ' ' };
                let token = if index % 2 == 0 { '+' } else { '2' };
                buffer.write_char(sep).unwrap();
                buffer.write_char(token).unwrap();
            }
            if splits.len() % 2 == 1 {
                buffer.write_char('2').unwrap();
            }

            // check that the buffer is valid
            dbg!(&buffer);
            let tokens = Lexer::lex(Source::source(&buffer)).unwrap();
            let token_tree = Reader::read(tokens);
            dbg!(&token_tree);
            assert!(token_tree.is_ok());
            if let TokenTree::Block(block) = token_tree.unwrap() {
                assert_eq!(block.len(), 1);
            }
        }
    }

    #[test]
    fn form_with_group() {
        let source = Source::source("print (1 + 2)");
        let tokens = Lexer::lex(source).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_ok())
    }

    #[test]
    fn list() {
        let source = Source::source("[a, b, c]");
        let tokens = Lexer::lex(source).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_ok())
    }

    /// Should return a syntax error - mismatched delimiters
    #[test]
    fn unbalanced_delims() {
        let source = Source::source("([)]");
        let tokens = Lexer::lex(source).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_err());
    }

    #[test]
    fn balanced_delims() {
        let source = Source::source("[(),[],{()[]}]{([][]){}}");
        let tokens = Lexer::lex(source).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_ok())
    }

    #[test]
    fn multiline_operators() {
        let source = Source::source("1 ++\n2 --\n 3");
        let tokens = Lexer::lex(source).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_ok())
    }

    #[test]
    fn unclosed_opening_paren() {
        let source = Source::source("(");
        let tokens = Lexer::lex(source).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_err());
    }

    #[test]
    fn unclosed_closing_paren() {
        let source = Source::source(")");
        let tokens = Lexer::lex(source).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_err());
    }

    #[test]
    fn multiline_block() {
        let tokens = Lexer::lex(Source::source("2\n+2")).unwrap();
        let token_tree = Reader::read(tokens);
        dbg!(&token_tree);
        assert!(token_tree.is_ok());
        if let TokenTree::Block(block) = token_tree.unwrap() {
            assert_eq!(block.len(), 1);
        }
    }

    #[test]
    fn multiline_form() {
        let source = Source::source("(\n2 \n+ 2\n)");
        let tokens = Lexer::lex(source).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_ok())
    }
}
