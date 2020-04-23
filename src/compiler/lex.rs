use crate::utils::annotation::Ann;
use crate::pipeline::token::{Token, AnnToken};

// A lexer parses a source (string) into a stream of tokens
// The Lexer struct essentially sanitizes the source,
// then asks the tokenizer to identify the next token.

// TODO: error handling, rather than just returning 'None'

struct Lexer {
    source: &'static str,
    offset: usize,
    tokens: Vec<AnnToken>,
}

impl Lexer {
    pub fn new(source: &'static str) -> Lexer {
        Lexer {
            source: source,
            offset: 0,
            tokens: vec![],
        }
    }

    fn all(&mut self) -> Option<()> {
        self.strip();

        while self.source.len() > self.offset {
            self.step()?;
        }

        Some(())
    }

    fn remaining(&self) -> &str {
        &self.source[self.offset..]
    }

    fn step(&mut self) -> Option<()> {
        // strip preceeding whitespace, get next token kind, build token
        let (kind, consumed) = Token::from(self.remaining())?;
        let token = AnnToken::new(kind, Ann::new(self.offset, consumed));

        self.offset += consumed;
        self.tokens.push(token);
        self.strip();

        return Some(());
    }

    fn strip(&mut self) {
        let mut len = 0;

        for char in self.remaining().chars() {
            // \n indicates a token, so it isn't 'whitespace'
            if !char.is_whitespace() || char == '\n' {
                break;
            }
            len += 1;
        }

        self.offset += len;
    }
}

pub fn lex(source: &'static str) -> Option<Vec<AnnToken>> {
    let mut lexer = Lexer::new(&source);

    // It's pretty self-explanatory
    // lex the whole source
    lexer.all()?;

    return Some(lexer.tokens);
}

// TODO: cfg test isn't working, so using #[test] for now
#[cfg(test)]
mod test {
    use super::*;
    use crate::vm::data::Data;
    use crate::vm::local::Local;

    // NOTE: lexing individual tokens is tested in pipeline::token

    #[test]
    fn lex_empty() {
        // no source code? no tokens!
        assert_eq!(lex(""), Some(vec![]));
    }

    #[test]
    fn lex_assignment() {
        let source = "heck = true";

        let result = vec![
            AnnToken::new(Token::Symbol(Local::new("heck".to_string())), Ann::new(0, 4)),
            AnnToken::new(Token::Assign,                                 Ann::new(5, 1)),
            AnnToken::new(Token::Boolean(Data::Boolean(true)),           Ann::new(7, 4)),
        ];

        assert_eq!(lex(source), Some(result));
    }

    #[test]
    fn whitespace() {
        let source = "  true  ;  ";

        let result = vec![
            AnnToken::new(Token::Boolean(Data::Boolean(true)), Ann::new(2, 4)),
            AnnToken::new(Token::Sep,                          Ann::new(8, 1)),
        ];

        assert_eq!(lex(source), Some(result));
    }

    #[test]
    fn block() {
        let source = "{\n\thello = true\n\thello\n}";

        // TODO: finish test

        let result = vec![
            AnnToken::new(Token::OpenBracket,                             Ann::new(0, 1)),
            AnnToken::new(Token::Sep,                                     Ann::new(1, 1)),
            AnnToken::new(Token::Symbol(Local::new("hello".to_string())), Ann::new(3, 5)),
            AnnToken::new(Token::Assign,                                  Ann::new(9, 1)),
            AnnToken::new(Token::Boolean(Data::Boolean(true)),            Ann::new(11, 4)),
            AnnToken::new(Token::Sep,                                     Ann::new(15, 1)),
            AnnToken::new(Token::Symbol(Local::new("hello".to_string())), Ann::new(17, 5)),
            AnnToken::new(Token::Sep,                                     Ann::new(22, 1)),
            AnnToken::new(Token::CloseBracket,                            Ann::new(23, 1)),
        ];

        assert_eq!(lex(source), Some(result));
    }
}
