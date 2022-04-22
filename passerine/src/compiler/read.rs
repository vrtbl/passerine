use crate::{
    common::span::{
        Span,
        Spanned,
    },
    compiler::syntax::{
        Note,
        Syntax,
    },
    construct::token::{
        Delim,
        Token,
        TokenTree,
        TokenTrees,
        Tokens,
    },
};

pub struct Reader {
    tokens:  Spanned<Tokens>,
    index:   usize,
    // stack of nested groupings
    opening: Vec<Spanned<Delim>>,
}

// TODO: return Token

impl Reader {
    pub fn read(tokens: Spanned<Tokens>) -> Result<Spanned<TokenTree>, Syntax> {
        let mut reader = Reader {
            tokens,
            index: 0,
            opening: vec![],
        };

        let result = reader.block()?;

        // if there are still unclosed delimiters on the opening
        // stack
        if !reader.opening.is_empty() {
            let still_opened = reader.opening.last().unwrap();
            Err(Syntax::error(
                &format!("Unclosed opening {}", still_opened.item,),
                &still_opened.span,
            ))
        } else {
            Ok(result)
        }
    }

    /// Returns the next token, advancing the lexer by 1.
    fn next_token(&mut self) -> Option<Spanned<Token>> {
        if self.index < self.tokens.item.len() {
            let token = &self.tokens.item[self.index];
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

    fn form(&mut self) -> Result<Spanned<TokenTrees>, Syntax> {
        let mut tokens: TokenTrees = vec![];

        let entire_span = loop {
            let token = match self.next_token() {
                Some(t) => t,
                None => {
                    // TODO: unwrap to the end of source span?
                    return Err(Syntax::error(
                        "Unexpected end of source while parsing form",
                        &Spanned::build(&tokens)
                            .unwrap_or_else(|| self.tokens.span.clone()),
                    ));
                },
            };

            let span = token.span;
            let item = match token.item {
                Token::Open(delim) => {
                    self.enter_group(Spanned::new(delim, span.clone()))?
                },
                Token::Close(delim) => {
                    let span = self.exit_group(Spanned::new(delim, span))?;
                    break span;
                },
                Token::Sep => continue,

                // Trivial conversion
                other => Spanned::new(Self::trivial(other).unwrap(), span),
            };

            tokens.push(item);
        };

        Ok(Spanned::new(tokens, entire_span))
    }

    fn block(&mut self) -> Result<Spanned<TokenTree>, Syntax> {
        let mut lines: Vec<Spanned<TokenTrees>> = vec![];
        let mut line: TokenTrees = vec![];
        let mut after_sep = false;
        let mut after_op = false;

        let entire_span = loop {
            let token = match self.next_token() {
                Some(t) => t,
                // We didn't hit a closing `}`, so this must either be the main
                // body, or we're missing a closing `}`. The missing closing `}`
                // is handled by `exit_group`, so we just need to break with a
                // realistic span.
                // TODO: get a realistic span!
                None => break self.tokens.span.clone(),
            };

            let span = token.span;
            let item = match token.item {
                Token::Open(delim) => {
                    self.enter_group(Spanned::new(delim, span.clone()))?
                },
                Token::Close(delim) => {
                    let span = self.exit_group(Spanned::new(delim, span))?;
                    break span;
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
                other => Spanned::new(Self::trivial(other).unwrap(), span),
            };

            if after_sep && !after_op && !line.is_empty() {
                // we can unwrap because we checked the line isn't empty
                let line_span = Spanned::build(&line).unwrap();
                let spanned_line = Spanned::new(line, line_span);
                lines.push(spanned_line);
                line = vec![];
            }

            after_sep = false;
            after_op = false;
            line.push(item);
        };

        if !line.is_empty() {
            let line_span = Spanned::build(&line).unwrap();
            let spanned_line = Spanned::new(line, line_span);
            lines.push(spanned_line);
        }

        Ok(Spanned::new(TokenTree::Block(lines), entire_span))
    }

    fn enter_group(
        &mut self,
        delim: Spanned<Delim>,
    ) -> Result<Spanned<TokenTree>, Syntax> {
        self.opening.push(delim.clone());

        match delim.item {
            Delim::Curly => Ok(self.block()?),
            Delim::Paren => self.form()?.map(|x| Ok(TokenTree::Form(x))),
            Delim::Square => self.form()?.map(|x| Ok(TokenTree::List(x))),
        }
    }

    fn exit_group(
        &mut self,
        closing_delim: Spanned<Delim>,
    ) -> Result<Span, Syntax> {
        let opening_delim = self.opening.pop().ok_or_else(|| {
            Syntax::error(
                &format!("Unexpected closing {}", closing_delim.item),
                &closing_delim.span,
            )
        })?;

        if opening_delim.item == closing_delim.item {
            return Ok(Span::combine(&opening_delim.span, &closing_delim.span));
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

    use proptest::prelude::*;

    use super::*;
    use crate::{
        common::source::Source,
        compiler::lex::Lexer,
    };

    /// Generates a source file from some tokens.
    /// Replaces each token with a minimal representative
    /// token. for example, all delimiters become curly
    /// brackets, all identifiers become `x`, and so on.
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

    /// Checks if there are a matching number of opening and
    /// closing delims. Doesn't care if delims are of
    /// different types To be used with
    /// `generate_minimal_source`.
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
            // dbg!(&source);
            let tokens = Lexer::lex(Source::source(&source)).unwrap();
            let token_tree = Reader::read(tokens);

            if balanced {
                prop_assert!(token_tree.is_ok())
            } else {
                prop_assert!(token_tree.is_err())
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
            prop_assert!(token_tree.is_ok());
            if let TokenTree::Block(block) = token_tree.unwrap().item {
                prop_assert_eq!(block.len(), 1);
            }
        }
    }

    #[test]
    fn double_open() {
        let tokens = Lexer::lex(Source::source("{{")).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_err());
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
        if let TokenTree::Block(block) = token_tree.unwrap().item {
            assert_eq!(block.len(), 1);
        }
    }

    #[test]
    fn multiline_form() {
        let source = Source::source("(\n2 \n+ 2\n)");
        let tokens = Lexer::lex(source).unwrap();
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_ok());
        println!("{}", token_tree.unwrap().span);
    }
}
