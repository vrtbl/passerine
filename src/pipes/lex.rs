use crate::utils::annotation::Ann;
use crate::pipeline::token::{Token, AnnotatedToken};

// idk, maybe make some sort of pratt parser? - nah
// a lexer is really just a set of rules to turn a string into a token
// it chooses the longest string
// so all we need is a rule-set
// ok, so rule-set has been defined in pipeline::token.rs

// TODO: error handling, rather than just returning 'None'
// TODO: I feel like there could be some more elegant separation

struct Lexer {
    source: &'static str,
    offset: usize,
    tokens: Vec<AnnotatedToken>,
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
        let token = AnnotatedToken::new(kind, Ann::new(&self.source, self.offset, consumed));

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

pub fn lex(source: &'static str) -> Option<Vec<AnnotatedToken>> {
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

    // lexing individual tokens is tested in pipeline::token

    #[test]
    fn lex_empty() {
        // no source code? no tokens!
        assert_eq!(lex(""), Some(vec![]));
    }

    #[test]
    fn lex_assignment() {
        let source = "heck = true";

        let result = vec![
            (Token::Symbol,  Ann::new(source, 0, 4)),
            (Token::Assign,  Ann::new(source, 5, 1)),
            (Token::Boolean, Ann::new(source, 7, 4)),
        ];

        assert_eq!(lex(source), Some(result));
    }

    #[test]
    fn whitespace() {
        let source = "  true  ;  ";

        let result = vec![
            (Token::Boolean, Ann::new(source, 2, 4)),
            (Token::Sep,     Ann::new(source, 8, 1)),
        ];

        assert_eq!(lex(source), Some(result));
    }
}
