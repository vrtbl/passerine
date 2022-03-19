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

        let result = reader.block();

        // if there are still unclosed delimiters on the opening stack
        if reader.opening.len() > 0 {
            let (_index, still_opened) = reader.opening.last().unwrap();
            let error = Syntax::error(
                &format!(
                    "Unclosed {}", still_opened.item,
                ), &still_opened.span);

            Err(error)
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
                    if !after_op && line.len() > 0 {
                        lines.push(line);
                        line = vec![];
                    }
                    continue;
                },

                Token::Op(op) => {
                    let spanned = Spanned::new(TokenTree::Op(op), span);
                    line.push(spanned);
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

        lines.push(line);
        Ok(TokenTree::Block(lines))
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
    use super::*;
    use crate::common::source::Source;
    use crate::compiler::lex::Lexer;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn random_tokens(tokens: Token) {
            dbg!(tokens);
        }
    }

    #[test]
    fn example_test() {
        let source = Source::source("print (1 + 2)");
        let tokens = Lexer::lex(source).unwrap();
        dbg!(&tokens);
        let token_tree = Reader::read(tokens).unwrap();
        dbg!(token_tree);
    }


    // Hey, if you're here, you're here for some compiler hacking.
    // So I just wrote this code, but I haven't really tested this.
    // Your quest, should you choose to embark on it, is to write some tests,
    // and fix the places this pass falls short.

    // This pass takes a flat list of Tokens (found in src/contruct/token)
    // and produces a TokenTree, essentially a list of tokens where groups of delimiters are nested.

    // For example, the code:
    // print (1 + 2)
    // Produces the Tokens:
    // Identifier print
    // Open (
    // Literal 1
    // Operator +
    // Literal 1
    // Close )

    // Note how the parenthesis aren't matched yet.
    // This pass keeps track of parenthesis and other delimiters,
    // and builds nested lists of tokens:

    // For example, the code:
    // print (1 + 2)
    // Produces the TokenTree:
    // Identifier print
    // Form (
    //     Literal 1
    //     Operator +
    //     Literal 1
    // )

    // Note how the parenthesis have been replaced with a single form
    // containing another list of tokens.

    // Look at example test, and see if you can find any input that "breaks the parser"
    // You can also try writing a property based test to help find unit tests.

    // Good luck! let me know if you have any questions.

    #[test]
    fn list() {
        let source = Source::source("[a, b, c]");
        let tokens = Lexer::lex(source).unwrap();
        dbg!(&tokens);
        let token_tree = Reader::read(tokens).unwrap();
        dbg!(token_tree);
    }

    // Should return a syntax error - mismatched delimiters
    #[test]
    fn unbalanced_delims() {
        let source = Source::source("([)]");
        let tokens = Lexer::lex(source).unwrap();
        dbg!(&tokens);
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_err());
    }

    #[test]
    fn balanced_delims() {
        let source = Source::source("[(),[],{()[]}]{([][]){}}");
        let tokens = Lexer::lex(source).unwrap();
        dbg!(&tokens);
        let token_tree = Reader::read(tokens).unwrap();
        dbg!(token_tree);
    }

    #[test]
    fn multiline_operators() {
        let source = Source::source("1 ++\n2 --\n 3");
        let tokens = Lexer::lex(source).unwrap();
        dbg!(&tokens);
        let token_tree = Reader::read(tokens).unwrap();
        dbg!(token_tree);
    }

    #[test]
    fn unclosed_paren() {
        let source = Source::source("(");
        let tokens = Lexer::lex(source).unwrap();
        dbg!(&tokens);
        let token_tree = Reader::read(tokens);
        assert!(token_tree.is_err());
        dbg!(token_tree.err().unwrap());
    }

    // honestly, I'm not sure how this should be parsed, but I'm adding it anyway
    #[test]
    fn multiline_block() {
        let source = Source::source("\n2 \n+ \n2\n");
        let tokens = Lexer::lex(source).unwrap();
        dbg!(&tokens);
        let token_tree = Reader::read(tokens).unwrap();
        dbg!(token_tree);
    }

    // honestly, I'm not sure how this should be parsed, but I'm adding it anyway
    #[test]
    fn multiline_form() {
        let source = Source::source("(\n2 \n+ 2\n)");
        let tokens = Lexer::lex(source).unwrap();
        dbg!(&tokens);
        let token_tree = Reader::read(tokens).unwrap();
        dbg!(token_tree);
    }
}
