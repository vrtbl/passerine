use std::str::FromStr;
use std::f64;

use crate::pipeline::source::Source;
use crate::pipeline::token::Token;
use crate::utils::runtime::Syntax;
use crate::utils::span::{ Span, Spanned };
use crate::vm::local::Local;
use crate::vm::data::Data;

type Bite = (Token, usize);

pub fn lex<'a>(source: Source) -> Result<Vec<Spanned<'a, Token>>, Syntax<'a>> {
    let mut lexer = Lexer::new(source);
    return lexer.all();
}

/// This represents a lexer object.
/// A lexer takes a source file and lexes it into tokens.
struct Lexer {
    source: Source,
    offset: usize,
}

impl Lexer {
    pub fn new(source: Source) -> Lexer {
        Lexer { source, offset: 0 }
    }

    fn all(&mut self) -> Result<Vec<Spanned<Token>>, Syntax> {
        let tokens = vec![];

        while self.remaining().len() != 0 {
            // strip preceeding whitespace, get next token kind, build token
            let (kind, consumed) = match Lexer::step(self.remaining()) {
                Ok(k)  => k,
                Err(e) => return Err(
                    Syntax::error(&e, Span::point(&self.source, self.offset))
                ),
            };

            // annotate it
            tokens.push(Spanned::new(
                kind,
                Span::new(&self.source, self.offset, consumed),
            ));
            self.offset += consumed;
            self.strip();
        }

        return Ok(tokens);
    }

    fn remaining(&self) -> &str {
        return &self.source.contents[self.offset..]
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

    pub fn step(source: &str) -> Result<Bite, String> {
        let rules: Vec<Box<dyn Fn(&str) -> Result<Bite, String>>> = vec![
            // higher up in order = higher precedence
            // think 'or' as symbol or 'or' as operator
            // static
            Box::new(|s| Lexer::open_bracket(s) ),
            Box::new(|s| Lexer::close_bracket(s)),
            Box::new(|s| Lexer::open_paren(s)   ),
            Box::new(|s| Lexer::close_paren(s)  ),
            Box::new(|s| Lexer::assign(s)       ),
            Box::new(|s| Lexer::lambda(s)       ),

            // variants
            Box::new(|s| Lexer::sep(s)    ),
            Box::new(|s| Lexer::boolean(s)),

            // dynamic
            Box::new(|s| Lexer::real(s)  ),
            Box::new(|s| Lexer::string(s)),
            // Box::new(|s| Lexer::int(s)),

            // keep this @ the bottom, lmao
            Box::new(|s| Lexer::symbol(s) ),
        ];

        // maybe some sort of map reduce?
        let mut best = Err("Next token is not known in this context".to_string());

        // check longest
        for rule in &rules {
            if let Ok((k, c)) = rule(source) {
                match best {
                    Err(_)              => best = Ok((k, c)),
                    Ok((_, o)) if c > o => best = Ok((k, c)),
                    Ok(_)               => (),
                }
            }
        }

        return best;
    }

    // helpers
    fn expect(source: &str, literal: &str) -> Result<usize, String> {
        if literal.len() > source.len() {
            return Err("Unexpected EOF while lexing".to_string());
        }

        match &source[..literal.len()] {
            s if s == literal => Ok(literal.len()),
            _                 => Err(format!("Expected '{}'", source)),
        }
    }

    fn eat_digits(source: &str) -> Result<usize, String> {
        let mut len = 0;

        for char in source.chars() {
            match char {
                n if n.is_digit(10) => len += 1,
                _                   => break,
            }
        }

        return if len == 0 { Err("Expected digits".to_string()) } else { Ok(len) };
    }

    fn literal(source: &str, literal: &str, kind: Token) -> Result<Bite, String> {
        Ok((kind, Lexer::expect(source, literal)?))
    }

    // token classifiers

    fn symbol(source: &str) -> Result<Bite, String> {
        // for now, a symbol is one or more ascii alphanumerics
        // TODO: extend to full unicode
        let mut len = 0;

        for char in source.chars() {
            if !char.is_ascii_alphanumeric() {
                break;
            }
            len += 1;
        }

        return match len {
            0 => Err("Expected a symbol".to_string()),
            // TODO: make sure that symbol name is correct
            l => Ok((Token::Symbol(Local::new(source[..l].to_string())), l)),
        };
    }

    fn open_bracket(source: &str) -> Result<Bite, String> {
        Lexer::literal(source, "{", Token::OpenBracket)
    }

    fn close_bracket(source: &str) -> Result<Bite, String> {
        Lexer::literal(source, "}", Token::CloseBracket)
    }

    fn open_paren(source: &str) -> Result<Bite, String> {
        Lexer::literal(source, "(", Token::OpenParen)
    }

    fn close_paren(source: &str) -> Result<Bite, String> {
        Lexer::literal(source, ")", Token::CloseParen)
    }

    fn assign(source: &str) -> Result<Bite, String> {
        Lexer::literal(source, "=", Token::Assign)
    }

    fn lambda(source: &str) -> Result<Bite, String> {
        Lexer::literal(source, "->", Token::Lambda)
    }

    fn real(source: &str) -> Result<Bite, String> {
        // TODO: NaNs, Infinity, the whole shebang
        // look at how f64::from_str is implemented, maybe?
        let mut len = 0;

        // one or more digits followed by a '.' followed by 1 or more digits
        len += Lexer::eat_digits(source)?;
        len += Lexer::expect(&source[len..], ".")?;
        len += Lexer::eat_digits(&source[len..])?;

        let number = match f64::from_str(&source[..len]) {
            Ok(n)  => n,
            Err(_) => panic!("Could not convert source to supposed real")
        };

        return Ok((Token::Number(Data::Real(number)), len));
    }

    // the below pattern is pretty common...
    // but I'm not going to abstract it out, yet

    fn string(source: &str) -> Result<Bite, String> {
        // TODO: read through the rust compiler and figure our how they do this
        // look into parse_str_lit

        let mut len    = 0;
        let mut escape = false;
        let mut string = "".to_string();

        len += Lexer::expect(source, "\"")?;

        for c in source[len..].chars() {
            len += 1;
            if escape {
                escape = false;
                // TODO: add more escape codes
                string.push(match c {
                    '\"' => '\"',
                    '\\' => '\\',
                    'n'  => '\n',
                    't'  => '\t',
                    o    => return Err(format!("Unknown escape code '\\{}'", o)),
                })
            } else {
                match c {
                    '\\' => escape = true,
                    '\"' => return Ok((Token::String(Data::String(string)), len)),
                    c    => string.push(c),
                }
            }
        }

        return Err("Unexpected EOF while parsing string literal".to_string());
    }

    fn boolean(source: &str) -> Result<Bite, String> {
        for (lit, val) in [
            ("true",  true),
            ("false", false),
        ].into_iter() {
            if let x @ Ok(_) = Lexer::literal(
                source, lit, Token::Boolean(Data::Boolean(*val))
            ) { return x; }
        }

        return Err("Expected a boolean".to_string());
    }

    fn sep(source: &str) -> Result<Bite, String> {
        match source.chars().next() {
            Some('\n') | Some(';') => Ok((Token::Sep, 1)),
            Some(_) => Err("Expected a separator, such as a newline".to_string()),
            None    => Err("Unexpected EOF while lexing".to_string()),
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::vm::data::Data;
//     use crate::vm::local::Local;
//
//     // NOTE: lexing individual tokens is tested in pipeline::token
//
//     #[test]
//     fn lex_empty() {
//         // no source code? no tokens!
//         assert_eq!(lex(Source::source("")), Ok(Vec::new()));
//     }
//
//     #[test]
//     fn lex_assignment() {
//         let source = Source::source("heck = true");
//
//         let result = vec![
//             Spanned::new(Token::Symbol(Local::new("heck".to_string())), Span::new(&source, 0, 4)),
//             Spanned::new(Token::Assign,                                 Span::new(&source, 5, 1)),
//             Spanned::new(Token::Boolean(Data::Boolean(true)),           Span::new(&source, 7, 4)),
//         ];
//
//         assert_eq!(lex(source), Ok(result));
//     }
//
//     #[test]
//     fn whitespace() {
//         let source = Source::source("  true  ;  ");
//
//         let result = vec![
//             Spanned::new(Token::Boolean(Data::Boolean(true)), Span::new(&source, 2, 4)),
//             Spanned::new(Token::Sep,                          Span::new(&source, 8, 1)),
//         ];
//
//         assert_eq!(lex(source), Ok(result));
//     }
//
//     #[test]
//     fn block() {
//         let source = Source::source("{\n\thello = true\n\thello\n}");
//
//         // TODO: finish test
//
//         let result = vec![
//             Spanned::new(Token::OpenBracket,                             Span::new(&source, 0, 1)),
//             Spanned::new(Token::Sep,                                     Span::new(&source, 1, 1)),
//             Spanned::new(Token::Symbol(Local::new("hello".to_string())), Span::new(&source, 3, 5)),
//             Spanned::new(Token::Assign,                                  Span::new(&source,  9, 1)),
//             Spanned::new(Token::Boolean(Data::Boolean(true)),            Span::new(&source, 11, 4)),
//             Spanned::new(Token::Sep,                                     Span::new(&source, 15, 1)),
//             Spanned::new(Token::Symbol(Local::new("hello".to_string())), Span::new(&source, 17, 5)),
//             Spanned::new(Token::Sep,                                     Span::new(&source, 22, 1)),
//             Spanned::new(Token::CloseBracket,                            Span::new(&source, 23, 1)),
//         ];
//
//         assert_eq!(lex(source), Ok(result));
//     }
//
//     #[test]
//     fn function() {
//         let source = Source::source("identity = x -> x\nidentity (identity \"heck\")");
//         let result = vec![
//             Spanned::new(Token::Symbol(Local::new("identity".to_string())), Span::new(&source, 0, 8)),
//             Spanned::new(Token::Assign,                                     Span::new(&source, 9, 1)),
//             Spanned::new(Token::Symbol(Local::new("x".to_string())),        Span::new(&source, 11, 1)),
//             Spanned::new(Token::Lambda,                                     Span::new(&source, 13, 2)),
//             Spanned::new(Token::Symbol(Local::new("x".to_string())),        Span::new(&source, 16, 1)),
//             Spanned::new(Token::Sep,                                        Span::new(&source, 17, 1)),
//             Spanned::new(Token::Symbol(Local::new("identity".to_string())), Span::new(&source, 18, 8)),
//             Spanned::new(Token::OpenParen,                                  Span::new(&source, 27, 1)),
//             Spanned::new(Token::Symbol(Local::new("identity".to_string())), Span::new(&source, 28, 8)),
//             Spanned::new(Token::String(Data::String("heck".to_string())),   Span::new(&source, 37, 6)),
//             Spanned::new(Token::CloseParen,                                 Span::new(&source, 43, 1)),
//         ];
//
//         assert_eq!(lex(source), Ok(result));
//     }
//
//     // each case tests the detection of a specific token type
//
//     #[test]
//     fn boolean() {
//         assert_eq!(
//             Token::from("true"),
//             Ok((Token::Boolean(Data::Boolean(true)), 4)),
//         );
//
//         assert_eq!(
//             Token::from("false"),
//             Ok((Token::Boolean(Data::Boolean(false)), 5)),
//         );
//     }
//
//     #[test]
//     fn assign() {
//         assert_eq!(
//             Token::from("="),
//             Ok((Token::Assign, 1)),
//         );
//     }
//
//     #[test]
//     fn symbol() {
//         assert_eq!(
//             Token::from("heck"),
//             Ok((Token::Symbol(Local::new("heck".to_string())), 4))
//         );
//     }
//
//     #[test]
//     fn sep() {
//         assert_eq!(
//             Token::from("\nheck"),
//             Ok((Token::Sep, 1)),
//         );
//
//         assert_eq!(
//             Token::from("; heck"),
//             Ok((Token::Sep, 1)),
//         );
//     }
//
//     #[test]
//     fn real() {
//         assert_eq!(
//             Token::from("2.0"),
//             Ok((Token::Number(Data::Real(2.0)), 3)),
//         );
//
//         assert_eq!(
//             Token::from("210938.2221"),
//             Ok((Token::Number(Data::Real(210938.2221)), 11)),
//         );
//     }
//
//     #[test]
//     fn string() {
//         let source = "\"heck\"";
//         assert_eq!(
//             Token::from(&source),
//             Ok((Token::String(Data::String("heck".to_string())), source.len())),
//         );
//
//         let escape = "\"I said, \\\"Hello, world!\\\" didn't I?\"";
//         assert_eq!(
//             Token::from(&escape),
//             Ok((
//                 Token::String(Data::String("I said, \"Hello, world!\" didn't I?".to_string())),
//                 escape.len()
//             )),
//         );
//
//         // TODO: unicode support
//     }
// }
