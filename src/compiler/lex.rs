use std::{
    str::FromStr,
    f64,
    rc::Rc,
};

use crate::common::{
    source::Source,
    span::{Span, Spanned},
    data::Data,
};

use crate::construct::{
    module::ThinModule,
    token::{Delim, Token, Tokens},
};

use crate::compiler::{lower::Lower, syntax::Syntax};

type Bite = (Token, usize);

impl Lower for ThinModule<Rc<Source>> {
    type Out = ThinModule<Tokens>;

    /// Simple function that lexes a source file into a token stream.
    /// Exposes the functionality of the `Lexer`.
    fn lower(self) -> Result<Self::Out, Syntax> {
        let mut lexer = Lexer::new(&self.repr);
        return Ok(ThinModule::thin(lexer.all()?));
    }
}

/// This represents a lexer object.
/// A lexer takes a source file and lexes it into tokens.
/// Note that this struct should not be controlled manually,
/// use the `lex` function instead.
pub struct Lexer {
    /// A reference to the source being lexed.
    source: Rc<Source>,
    /// The current lexing offset.
    offset: usize,
    /// Closing tokens to match.
    closing: Vec<(Delim, usize)>,
}

impl Lexer {
    /// Create a new empty lexer.
    pub fn new(source: &Rc<Source>) -> Lexer {
        Lexer { source: Rc::clone(source), offset: 0, closing: vec![] }
    }

    /// Run the lexer, generating the entire token stream.
    pub fn all(&mut self) -> Result<Tokens, Syntax> {
        let mut tokens = vec![];

        while self.remaining().len() != 0 {
            // strip preceeding whitespace
            self.strip();

            // clear out comments
            self.offset += Lexer::comment(&self.remaining());
            self.offset += Lexer::multi_comment(&self.remaining());

            // strip leading whitespace
            self.strip();

            // current lexer error location, if we error
            let error_span = &Span::point(&self.source, self.offset);

            // check for delimiters
            // not a fan of continue, but oh well
            // who is, anyway?
            if let Some(delim) = Lexer::open_delim(self.remaining()) {
                self.closing.push((delim, tokens.len()));
                self.offset += 1;
                continue;
            } else if let Some(delim) = Lexer::close_delim(self.remaining()) {
                let back = match self.closing.pop() {
                    Some((d, back)) if d == delim => back,
                    _ => return Err(Syntax::error(
                        "Unexpected closing delimiter with no match",
                        error_span,
                    )),
                };

                let token_group = tokens.split_off(back);
                let group_span  = Spanned::build(&token_group);
                let group_token = Token::Group { delim, tokens: token_group };
                tokens.push(Spanned::new(group_token, group_span));

                self.offset += 1;
                continue;
            }

            // get next token kind, build token
            let (kind, consumed) = match self.step() {
                Ok(k)  => k,
                Err(e) => return Err(Syntax::error(&e, &error_span)),
            };

            // annotate it
            tokens.push(Spanned::new(
                kind,
                Span::new(&self.source, self.offset, consumed),
            ));
            self.offset += consumed;
        }

        tokens.push(Spanned::new(Token::End, Span::empty()));

        return Ok(tokens);
    }

    /// Step the lexer, returning the next token.
    pub fn step(&self) -> Result<Bite, String> {
        let source = self.remaining();

        let rules: Vec<Box<dyn Fn(&str) -> Result<Bite, String>>> = vec![
            // higher up in order = higher precedence
            Box::new(Lexer::sep),

            // words
            Box::new(Lexer::label),
            Box::new(Lexer::symbol),
            Box::new(Lexer::op),

            // data
            Box::new(Lexer::unit),
            Box::new(Lexer::boolean),
            Box::new(Lexer::real),
            Box::new(Lexer::integer),
            Box::new(Lexer::string),
        ];

        // maybe some sort of map reduce?
        let mut best = Err("Unexpected token".to_string());

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

    pub fn open_delim(source: &str) -> Option<Delim> {
        if let Some(c) = source.chars().next() {
            Some(match c {
                '(' => Delim::Paren,
                '{' => Delim::Curly,
                '[' => Delim::Square,
                _   => return None,
            })
        } else {
            None
        }
    }

    pub fn close_delim(source: &str) -> Option<Delim> {
        if let Some(c) = source.chars().next() {
            Some(match c {
                ')' => Delim::Paren,
                '}' => Delim::Curly,
                ']' => Delim::Square,
                _   => return None,
            })
        } else {
            None
        }
    }

    // helpers

    /// Helper function that returns the remaining source to be lexed as a `&str`.
    pub fn remaining(&self) -> &str {
        return &self.source.contents[self.offset..]
    }

    /// Helper function that Strips leading whitespace.
    /// Note that a newline is not leading whitespace, it's a separator token.
    pub fn strip(&mut self) {
        let mut len = 0;

        for char in self.remaining().chars() {
            // \n indicates a token, so it isn't 'whitespace'
            if !char.is_whitespace() || char == '\n' {
                break;
            }
            len += char.len_utf8();
        }

        self.offset += len;
    }

    /// Helper function that expects an exact literal.
    pub fn expect(source: &str, literal: &str) -> Result<usize, String> {
        if literal.len() > source.len() {
            return Err("Unexpected EOF while lexing".to_string());
        }

        match &source.as_bytes()[..literal.len()] {
            s if s == literal.as_bytes() => Ok(literal.len()),
            _                            => Err(format!("Expected '{}'", source)),
        }
    }

    /// Helper function that eats numeric digits,
    /// returning how many lead.
    pub fn eat_digits(source: &str) -> Result<usize, String> {
        let mut len = 0;

        for char in source.chars() {
            match char {
                n if n.is_digit(10) => len += n.len_utf8(),
                _                   => break,
            }
        }

        return if len == 0 { Err("Expected digits".to_string()) } else { Ok(len) };
    }

    /// Helper function that expects a literal, returning an error otherwise.
    pub fn literal(source: &str, literal: &str, kind: Token) -> Result<Bite, String> {
        Ok((kind, Lexer::expect(source, literal)?))
    }

    // token classifiers

    // TODO: refactor comment and multi-line for doc-comments

    /// Parses a single-line comment,
    /// which ignores from "--" until the next newline.
    pub fn comment(source: &str) -> usize {
        let mut len = match Lexer::expect(source, "--") {
            Ok(n) => n,
            Err(_) => { return 0; },
        };

        for char in source[len..].chars() {
            if char == '\n' { break; }
            len += char.len_utf8();
        }

        return len;
    }

    /// Parses a nestable multi-line comment,
    /// Which begins with `-{` and ends with `}-`.
    pub fn multi_comment(source: &str) -> usize {
        let mut len: usize = match Lexer::expect(source, "-{") {
            Ok(n) => n,
            Err(_) => { return 0; },
        };

        for char in source[len..].chars() {
            if let Ok(_) = Lexer::expect(&source[len..], "-{") {
                len += Lexer::multi_comment(&source[len..]);
            } else if let Ok(end) = Lexer::expect(&source[len..], "}-") {
                len += end; break;
            } else {
                len += char.len_utf8();
            }
        }

        return len;
    }

    /// Classifies a symbol or a label.
    /// A series of alphanumerics and certain ascii punctuation (see `Lexer::is_alpha`).
    /// Can not start with a numeric character.
    pub fn identifier(source: &str) -> Result<Bite, String> {
        let mut len = 0;

        for char in source.chars() {
            match char {
                a if a.is_alphanumeric()
                  || "_".contains(a)
                  => { len += a.len_utf8() },
                _ => { break;              },
            }
        }

        if len == 0 {
            return Err("Expected an alphanumeric character".to_string());
        }

        let contents = source[0..len].to_string();
        let first = source.chars().next().unwrap();
        match first {
            n if n.is_numeric() => Err(
                "Can not start with a numeric character".to_string()
            ),
            s if s.is_uppercase() => Ok((Token::Label(contents), len)), // label
            _ => Ok((Token::Iden(contents), len)), // symbol
        }
    }

    /// Classifies a symbol (i.e. variable name).
    pub fn symbol(source: &str) -> Result<Bite, String> {
        if let symbol @ (Token::Iden(_), _) = Lexer::identifier(source)? {
            Ok(symbol)
        } else {
            Err("Expected a symbol".to_string())
        }
    }

    /// Classifies a label (i.e. data wrapper).
    /// Must start with an uppercase character.
    pub fn label(source: &str) -> Result<Bite, String> {
        if let label @ (Token::Label(_), _) = Lexer::identifier(source)? {
            Ok(label)
        } else {
            Err("Expected a Label".to_string())
        }
    }

    pub fn op(source: &str) -> Result<Bite, String> {
        let mut len = 0;

        for char in source.chars() {
            match char {
                a if "!#$%&*+,-./:<=>?@\\^_`|~".contains(a)
                  => { len += a.len_utf8() },
                _ => { break;              },
            }
        }

        if len == 0 {
            return Err("Expected an operator".to_string());
        }

        let contents = source[0..len].to_string();
        Ok((Token::Op(contents), len))
    }

    /// Matches a number with a decimal point.
    pub fn real(source: &str) -> Result<Bite, String> {
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

        return Ok((Token::Data(Data::Real(number)), len));
    }

    pub fn integer(source: &str) -> Result<Bite, String> {
        let mut len = 0;
        len += Lexer::eat_digits(source)?;

        let number = match i64::from_str(&source[..len]) {
            Ok(n) => n,
            Err(_) => panic!("Could not convert source to supposed integer"),
        };

        // TODO: introduce new token?
        return Ok((Token::Data(Data::Integer(number)), len));
    }

    /// Matches a string, converting escapes.
    pub fn string(source: &str) -> Result<Bite, String> {
        // TODO: read through the rust compiler and figure our how they do this
        // look into parse_str_lit

        let mut len    = 0;
        let mut escape = false;
        let mut string = "".to_string();

        len += Lexer::expect(source, "\"")?;

        for c in source[len..].chars() {
            len += c.len_utf8();
            if escape {
                escape = false;
                // TODO: add more escape codes
                string.push(match c {
                    '"'  => '"',
                    '\\' => '\\',
                    'n'  => '\n',
                    't'  => '\t',
                    'r'  => '\r',
                    o    => return Err(format!("Unknown escape code '\\{}'", o)),
                })
            } else {
                match c {
                    '\\' => escape = true,
                    '\"' => return Ok((Token::Data(Data::String(string)), len)),
                    c    => string.push(c),
                }
            }
        }

        return Err("Unexpected EOF while parsing string literal".to_string());
    }

    /// Matches an empty tuple
    pub fn unit(source: &str) -> Result<Bite, String> {
        Lexer::literal(source, "()", Token::Data(Data::Unit))
    }

    // TODO: booleans implemented in std
    /// Matches a literal boolean.
    pub fn boolean(source: &str) -> Result<Bite, String> {
        for (lit, val) in [
            ("true",  true),
            ("false", false),
        ].iter() {
            if let x @ Ok(_) = Lexer::literal(
                source, lit, Token::Data(Data::Boolean(*val))
            ) { return x; }
        }

        return Err("Expected a boolean".to_string());
    }

    /// Matches a separator.
    /// Note that separators are special, as they're mostly ignored
    /// They're used to denote lines in functions blocks.
    /// A separator is either a newline or semicolon.
    /// They're grouped, so something like ';\n' is only one separator.
    /// Although the parser makes no assumptions,
    /// there should be only at most one separator
    /// between any two non-separator tokens.
    pub fn sep(source: &str) -> Result<Bite, String> {
        let mut chars = source.chars();
        let c = chars.next()
            .ok_or("Unexpected EOF while parsing")?;

        // a newline or a semicolon
        if c != '\n' && c != ';' {
            return Err("Expected a separator such as a newline or semicolon".to_string())
        }

        // followed by n semicolons/whitespace (including newline)
        let mut len = c.len_utf8();
        for c in chars {
            if c != ';' && !c.is_whitespace() {
                break;
            }
            len += c.len_utf8();
        }

        return Ok((Token::Sep, len));
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::common::data::Data;
//
//     // NOTE: lexing individual tokens is tested in pipeline::token
//
//     #[test]
//     fn empty() {
//         // no source code? no tokens!
//         let result = ThinModule::thin(Source::source("")).lower();
//         let target: Result<ThinModule<Tokens>, Syntax> =
//             Ok(ThinModule::thin(vec![Spanned::new(Token::End, Span::empty())]));
//
//         assert_eq!(result, target);
//     }
//
//     #[test]
//     fn assignment() {
//         let source = Source::source("heck = true");
//
//         let result = vec![
//             Spanned::new(Token::Symbol,                       Span::new(&source, 0, 4)),
//             Spanned::new(Token::Assign,                       Span::new(&source, 5, 1)),
//             Spanned::new(Token::Boolean(Data::Boolean(true)), Span::new(&source, 7, 4)),
//             Spanned::new(Token::End,                          Span::empty()),
//         ];
//
//         assert_eq!(ThinModule::thin(source).lower(), Ok(ThinModule::thin(result)));
//     }
//
//     #[test]
//     fn whitespace() {
//         let source = Source::source("  true  ;  ");
//
//         let result = vec![
//             Spanned::new(Token::Boolean(Data::Boolean(true)), Span::new(&source, 2, 4)),
//             Spanned::new(Token::Sep,                          Span::new(&source, 8, 3)),
//             Spanned::new(Token::End,                          Span::empty()),
//
//         ];
//
//         assert_eq!(ThinModule::thin(source).lower(), Ok(ThinModule::thin(result)));
//     }
//
//     #[test]
//     fn block() {
//         let source = Source::source("{\n\thello = true\n\thello\n}");
//
//         let result = vec![
//             Spanned::new(Token::OpenBracket,                  Span::new(&source, 0, 1)),
//             Spanned::new(Token::Sep,                          Span::new(&source, 1, 2)),
//             Spanned::new(Token::Symbol,                       Span::new(&source, 3, 5)),
//             Spanned::new(Token::Assign,                       Span::new(&source,  9, 1)),
//             Spanned::new(Token::Boolean(Data::Boolean(true)), Span::new(&source, 11, 4)),
//             Spanned::new(Token::Sep,                          Span::new(&source, 15, 2)),
//             Spanned::new(Token::Symbol,                       Span::new(&source, 17, 5)),
//             Spanned::new(Token::Sep,                          Span::new(&source, 22, 1)),
//             Spanned::new(Token::CloseBracket,                 Span::new(&source, 23, 1)),
//             Spanned::new(Token::End,                          Span::empty()),
//         ];
//
//         assert_eq!(ThinModule::thin(source).lower(), Ok(ThinModule::thin(result)));
//     }
//
//     #[test]
//     fn function() {
//         let source = Source::source("identity = x -> x\nidentity (identity \"heck\")");
//
//         let result = vec![
//             Spanned::new(Token::Symbol,                                   Span::new(&source, 0, 8)),
//             Spanned::new(Token::Assign,                                   Span::new(&source, 9, 1)),
//             Spanned::new(Token::Symbol,                                   Span::new(&source, 11, 1)),
//             Spanned::new(Token::Lambda,                                   Span::new(&source, 13, 2)),
//             Spanned::new(Token::Symbol,                                   Span::new(&source, 16, 1)),
//             Spanned::new(Token::Sep,                                      Span::new(&source, 17, 1)),
//             Spanned::new(Token::Symbol,                                   Span::new(&source, 18, 8)),
//             Spanned::new(Token::OpenParen,                                Span::new(&source, 27, 1)),
//             Spanned::new(Token::Symbol,                                   Span::new(&source, 28, 8)),
//             Spanned::new(Token::String(Data::String("heck".to_string())), Span::new(&source, 37, 6)),
//             Spanned::new(Token::CloseParen,                               Span::new(&source, 43, 1)),
//             Spanned::new(Token::End,                                      Span::empty()),
//         ];
//
//         assert_eq!(ThinModule::thin(source).lower(), Ok(ThinModule::thin(result)));
//     }
//
//     // helper function for the following tests
//
//     fn test_literal(literal: &str, token: Token, length: usize) -> bool {
//         let result = Lexer::new(&Source::source(literal)).step();
//
//         match result {
//             Ok(v) => v == (token, length),
//             Err(_) => false
//         }
//     }
//
//     // each case tests the detection of a specific token type
//
//     #[test]
//     fn boolean() {
//         if !test_literal("true",  Token::Boolean(Data::Boolean(true)), 4)  { panic!() }
//         if !test_literal("false", Token::Boolean(Data::Boolean(false)), 5) { panic!() }
//     }
//
//     #[test]
//     fn assign() {
//         if !test_literal("=", Token::Assign, 1) { panic!() }
//     }
//
//     #[test]
//     fn symbol() {
//         if !test_literal("orchard", Token::Symbol, 7) { panic!() }
//     }
//
//     #[test]
//     fn sep() {
//         if !test_literal(
//             "\n  heck",
//             Token::Sep,
//             3,
//         ) { panic!() }
//
//         if !test_literal(
//             ";\n ; heck",
//             Token::Sep,
//             5,
//         ) { panic!() }
//     }
//
//     #[test]
//     fn real() {
//         if !test_literal(
//             "2.0",
//             Token::Number(Data::Real(2.0)),
//             3,
//         ) { panic!() }
//
//         if !test_literal(
//             "210938.2221",
//             Token::Number(Data::Real(210938.2221)),
//             11,
//         ) { panic!() }
//     }
//
//     #[test]
//     fn string() {
//         let source = "\"heck\"";
//         if !test_literal(
//             source,
//             Token::String(Data::String("heck".to_string())),
//             source.len(),
//         ) { panic!() }
//
//         let escape = "\"I said, \\\"Hello, world!\\\" didn't I?\"";
//         if !test_literal(
//             escape,
//             Token::String(Data::String("I said, \"Hello, world!\" didn't I?".to_string())),
//             escape.len(),
//         ) { panic!() }
//
//         let unicode = "\"Yo 👋! Ünícode µ works just fine 🚩! うん、気持ちいい！\"";
//         if !test_literal(
//             unicode,
//             Token::String(Data::String("Yo 👋! Ünícode µ works just fine 🚩! うん、気持ちいい！".to_string())),
//             unicode.len(),
//         ) { panic!() }
//     }
//
//     #[test]
//     fn comma() {
//         let source = Source::source("heck\\ man");
//         let tokens = ThinModule::thin(source.clone()).lower();
//         assert_eq!(tokens, Err(Syntax::error("Unexpected token", &Span::new(&source, 4, 0))));
//     }
// }
