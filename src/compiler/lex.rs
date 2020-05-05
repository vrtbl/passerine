use crate::pipeline::source::Source;
use crate::pipeline::token::{Token, AnnToken};
use crate::utils::error::CompilerError;
use crate::utils::annotation::Ann;

// A lexer parses a source (string) into a stream of tokens
// The Lexer struct essentially sanitizes the source,
// then asks the tokenizer to identify the next token.

pub fn lex<'a>(source: Source) -> Result<Vec<AnnToken<'a>>, CompilerError<'a>> {
    let mut lexer = Lexer::new(source);

    // It's pretty self-explanatory
    // lex the whole source
    match lexer.all() {
        None    => (),
        Some(e) => return Err(e),
    }

    return Ok(lexer.tokens);
}

struct Lexer<'a> {
    source: Source,
    offset: usize,
    tokens: Vec<AnnToken<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: Source) -> Lexer<'a> {
        Lexer {
            source,
            offset: 0,
            tokens: vec![],
        }
    }

    fn all(&'a mut self) -> Option<CompilerError<'a>> {
        self.strip();

        while self.source.contents.len() > self.offset {
            self.step()?;
        }

        return None;
    }

    fn remaining(&self) -> &str {
        &self.source.contents[self.offset..]
    }

    fn step(&'a mut self) -> Option<CompilerError<'a>> {
        // strip preceeding whitespace, get next token kind, build token
        let (kind, consumed) = match Token::from(self.remaining()) {
            Ok(k)  => k,
            Err(e) => return Some(
                CompilerError::Syntax(&e, Ann::new(&self.source, self.offset, 1))
            ),
        };
        let token = AnnToken::new(kind, Ann::new(&self.source, self.offset, consumed));

        self.offset += consumed;
        self.tokens.push(token);
        self.strip();

        return None;
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
        assert_eq!(lex(Source::source("")), Ok(vec![]));
    }

    #[test]
    fn lex_assignment() {
        let source = Source::source("heck = true");

        let result = vec![
            AnnToken::new(Token::Symbol(Local::new("heck".to_string())), Ann::new(&source, 0, 4)),
            AnnToken::new(Token::Assign,                                 Ann::new(&source, 5, 1)),
            AnnToken::new(Token::Boolean(Data::Boolean(true)),           Ann::new(&source, 7, 4)),
        ];

        assert_eq!(lex(source), Ok(result));
    }

    #[test]
    fn whitespace() {
        let source = Source::source("  true  ;  ");

        let result = vec![
            AnnToken::new(Token::Boolean(Data::Boolean(true)), Ann::new(&source, 2, 4)),
            AnnToken::new(Token::Sep,                          Ann::new(&source, 8, 1)),
        ];

        assert_eq!(lex(source), Ok(result));
    }

    #[test]
    fn block() {
        let source = Source::source("{\n\thello = true\n\thello\n}");

        // TODO: finish test

        let result = vec![
            AnnToken::new(Token::OpenBracket,                             Ann::new(&source, 0, 1)),
            AnnToken::new(Token::Sep,                                     Ann::new(&source, 1, 1)),
            AnnToken::new(Token::Symbol(Local::new("hello".to_string())), Ann::new(&source, 3, 5)),
            AnnToken::new(Token::Assign,                                  Ann::new(&source,  9, 1)),
            AnnToken::new(Token::Boolean(Data::Boolean(true)),            Ann::new(&source, 11, 4)),
            AnnToken::new(Token::Sep,                                     Ann::new(&source, 15, 1)),
            AnnToken::new(Token::Symbol(Local::new("hello".to_string())), Ann::new(&source, 17, 5)),
            AnnToken::new(Token::Sep,                                     Ann::new(&source, 22, 1)),
            AnnToken::new(Token::CloseBracket,                            Ann::new(&source, 23, 1)),
        ];

        assert_eq!(lex(source), Ok(result));
    }

    #[test]
    fn function() {
        let source = Source::source("identity = x -> x\nidentity (identity \"heck\")");
        let result = vec![
            AnnToken::new(Token::Symbol(Local::new("identity".to_string())), Ann::new(&source, 0, 8)),
            AnnToken::new(Token::Assign,                                     Ann::new(&source, 9, 1)),
            AnnToken::new(Token::Symbol(Local::new("x".to_string())),        Ann::new(&source, 11, 1)),
            AnnToken::new(Token::Lambda,                                     Ann::new(&source, 13, 2)),
            AnnToken::new(Token::Symbol(Local::new("x".to_string())),        Ann::new(&source, 16, 1)),
            AnnToken::new(Token::Sep,                                        Ann::new(&source, 17, 1)),
            AnnToken::new(Token::Symbol(Local::new("identity".to_string())), Ann::new(&source, 18, 8)),
            AnnToken::new(Token::OpenParen,                                  Ann::new(&source, 27, 1)),
            AnnToken::new(Token::Symbol(Local::new("identity".to_string())), Ann::new(&source, 28, 8)),
            AnnToken::new(Token::String(Data::String("heck".to_string())),   Ann::new(&source, 37, 6)),
            AnnToken::new(Token::CloseParen,                                 Ann::new(&source, 43, 1)),
        ];

        assert_eq!(lex(source), Ok(result));
    }
}
