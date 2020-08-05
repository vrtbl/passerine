use std::{
    str::FromStr,
    f64,
    rc::Rc,
};

use crate::common::{
    source::Source,
    span::{ Span, Spanned },
    data::Data,
    local::Local,
};

use crate::compiler::{
    token::Token,
    syntax::Syntax,
};

type Bite = (Token, usize);

pub fn lex(source: Rc<Source>) -> Result<Vec<Spanned<Token>>, Syntax> {
    let mut lexer = Lexer::new(&source);
    return lexer.all();
}

/// This represents a lexer object.
/// A lexer takes a source file and lexes it into tokens.
struct Lexer {
    source: Rc<Source>,
    offset: usize,
}

impl Lexer {
    pub fn new(source: &Rc<Source>) -> Lexer {
        Lexer { source: Rc::clone(source), offset: 0 }
    }

    fn all(&mut self) -> Result<Vec<Spanned<Token>>, Syntax> {
        let mut tokens = vec![];

        while self.remaining().len() != 0 {
            // strip preceeding whitespace
            self.strip();

            // get next token kind, build token
            let (kind, consumed) = match self.step() {
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
        }

        return Ok(tokens);
    }

    pub fn step(&self) -> Result<Bite, String> {
        let source = self.remaining();

        let rules: Vec<Box<dyn Fn(&str) -> Result<Bite, String>>> = vec![
            // higher up in order = higher precedence
            // think 'or' as literal or 'or' as operator

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

    // helpers

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

    fn expect(source: &str, literal: &str) -> Result<usize, String> {
        if literal.len() > source.len() {
            return Err("Unexpected EOF while lexing".to_string());
        }

        match &source.as_bytes()[..literal.len()] {
            s if s == literal.as_bytes() => Ok(literal.len()),
            _                            => Err(format!("Expected '{}'", source)),
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
            // TODO: give symbol access to the index
            l => Ok((Token::Symbol(Span::empty()), l)),
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

    fn string(source: &str) -> Result<Bite, String> {
        // TODO: read through the rust compiler and figure our how they do this
        // look into parse_str_lit

        let mut len    = 0;
        let mut escape = false;
        let mut string = "".to_string();

        len += Lexer::expect(source, "\"")?;

        for c in source[len..].chars() {
            print!("{}|", c);
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
        ].iter() {
            if let x @ Ok(_) = Lexer::literal(
                source, lit, Token::Boolean(Data::Boolean(*val))
            ) { return x; }
        }

        return Err("Expected a boolean".to_string());
    }

    fn sep(source: &str) -> Result<Bite, String> {
        let mut chars = source.chars();
        let c = chars.next()
            .ok_or("Unexpected EOF while parsing")?;

        // a newline or a semicolon
        if c != '\n' && c != ';' {
            return Err("Expected a separator such as a newline or semicolon".to_string())
        }

        // followed by n semicolons/whitespace (including newline)
        let mut len = 1;
        for c in chars {
            if c != ';' && !c.is_whitespace() {
                break;
            }
            len += 1;
        }

        return Ok((Token::Sep, len));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::data::Data;
    use crate::common::local::Local;

    // NOTE: lexing individual tokens is tested in pipeline::token

    #[test]
    fn empty() {
        // no source code? no tokens!
        let result = lex(Source::source(""));
        let target: Result<Vec<Spanned<Token>>, Syntax> = Ok(vec![]);
        assert_eq!(result, target);
    }

    #[test]
    fn assignment() {
        let source = Source::source("heck = true");

        let result = vec![
            Spanned::new(Token::Symbol,                       Span::new(&source, 0, 4)),
            Spanned::new(Token::Assign,                       Span::new(&source, 5, 1)),
            Spanned::new(Token::Boolean(Data::Boolean(true)), Span::new(&source, 7, 4)),
        ];

        assert_eq!(lex(source), Ok(result));
    }

    #[test]
    fn whitespace() {
        let source = Source::source("  true  ;  ");

        let result = vec![
            Spanned::new(Token::Boolean(Data::Boolean(true)), Span::new(&source, 2, 4)),
            Spanned::new(Token::Sep,                          Span::new(&source, 8, 3)),
        ];

        assert_eq!(lex(source), Ok(result));
    }

    #[test]
    fn block() {
        let source = Source::source("{\n\thello = true\n\thello\n}");

        // TODO: finish test

        let result = vec![
            Spanned::new(Token::OpenBracket,                  Span::new(&source, 0, 1)),
            Spanned::new(Token::Sep,                          Span::new(&source, 1, 2)),
            Spanned::new(Token::Symbol,                       Span::new(&source, 3, 5)),
            Spanned::new(Token::Assign,                       Span::new(&source,  9, 1)),
            Spanned::new(Token::Boolean(Data::Boolean(true)), Span::new(&source, 11, 4)),
            Spanned::new(Token::Sep,                          Span::new(&source, 15, 2)),
            Spanned::new(Token::Symbol,                       Span::new(&source, 17, 5)),
            Spanned::new(Token::Sep,                          Span::new(&source, 22, 1)),
            Spanned::new(Token::CloseBracket,                 Span::new(&source, 23, 1)),
        ];

        assert_eq!(lex(source), Ok(result));
    }

    #[test]
    fn function() {
        let source = Source::source("identity = x -> x\nidentity (identity \"heck\")");
        let result = vec![
            Spanned::new(Token::Symbol,                                   Span::new(&source, 0, 8)),
            Spanned::new(Token::Assign,                                   Span::new(&source, 9, 1)),
            Spanned::new(Token::Symbol,                                   Span::new(&source, 11, 1)),
            Spanned::new(Token::Lambda,                                   Span::new(&source, 13, 2)),
            Spanned::new(Token::Symbol,                                   Span::new(&source, 16, 1)),
            Spanned::new(Token::Sep,                                      Span::new(&source, 17, 1)),
            Spanned::new(Token::Symbol,                                   Span::new(&source, 18, 8)),
            Spanned::new(Token::OpenParen,                                Span::new(&source, 27, 1)),
            Spanned::new(Token::Symbol,                                   Span::new(&source, 28, 8)),
            Spanned::new(Token::String(Data::String("heck".to_string())), Span::new(&source, 37, 6)),
            Spanned::new(Token::CloseParen,                               Span::new(&source, 43, 1)),
        ];

        assert_eq!(lex(source), Ok(result));
    }

    // helper function for the following tests

    fn test_literal(literal: &str, token: Token, length: usize) -> bool {
        let result = Lexer::new(&Source::source(literal)).step();

        println!("{:?}", result);

        match result {
            Ok(v) => v == (token, length),
            Err(_) => false
        }
    }

    // each case tests the detection of a specific token type

    #[test]
    fn boolean() {
        if !test_literal("true",  Token::Boolean(Data::Boolean(true)), 4)  { panic!() }
        if !test_literal("false", Token::Boolean(Data::Boolean(false)), 5) { panic!() }
    }

    #[test]
    fn assign() {
        if !test_literal("=", Token::Assign, 1) { panic!() }
    }

    #[test]
    fn symbol() {
        if !test_literal("orchard", Token::Symbol, 7) { panic!() }
    }

    #[test]
    fn sep() {
        if !test_literal(
            "\n  heck",
            Token::Sep,
            3,
        ) { panic!() }

        if !test_literal(
            ";\n ; heck",
            Token::Sep,
            5,
        ) { panic!() }
    }

    #[test]
    fn real() {
        if !test_literal(
            "2.0",
            Token::Number(Data::Real(2.0)),
            3,
        ) { panic!() }

        if !test_literal(
            "210938.2221",
            Token::Number(Data::Real(210938.2221)),
            11,
        ) { panic!() }
    }

    #[test]
    fn string() {
        let source = "\"heck\"";
        if !test_literal(
            source,
            Token::String(Data::String("heck".to_string())),
            source.len(),
        ) { panic!() }

        let escape = "\"I said, \\\"Hello, world!\\\" didn't I?\"";
        if !test_literal(
            escape,
            Token::String(Data::String("I said, \"Hello, world!\" didn't I?".to_string())),
            escape.len(),
        ) { panic!() }

        let unicode = "\"Yo 👋! Ünícode µ works just fine 🚩! うん、気持ちいい！\"";
        println!("{}", unicode.len());
        if !test_literal(
            unicode,
            Token::String(Data::String("Yo 👋! Ünícode µ works just fine 🚩! うん、気持ちいい！".to_string())),
            unicode.chars().collect::<Vec<char>>().len(),
        ) { panic!() }
    }
}
