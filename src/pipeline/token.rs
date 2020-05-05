use std::str::FromStr;
use std::f64;

use crate::utils::annotation::Ann;
use crate::vm::data::Data;
use crate::vm::local::Local;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    // Delimiters
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    Sep,

    Assign,
    Lambda,

    // Datatypes
    Symbol(Local),
    Number(Data),
    String(Data),
    Boolean(Data),
}

type Consume = Result<(Token, usize), String>;

impl Token {
    pub fn from(source: &str) -> Consume {
        // check all functions
        // but are closures really the way to go?
        // also, maybe use array rather than vec?
        // also, there's no gaurantee I remember to closure-wrap all the functions
        // probably a more idiomatic way tbh
        let rules: Vec<Box<dyn Fn(&str) -> Consume>> = vec![
            // higher up in order = higher precedence
            // think 'or' as symbol or 'or' as operator
            // static
            Box::new(|s| Token::open_bracket(s) ),
            Box::new(|s| Token::close_bracket(s)),
            Box::new(|s| Token::open_paren(s)   ),
            Box::new(|s| Token::close_paren(s)  ),
            Box::new(|s| Token::assign(s)       ),
            Box::new(|s| Token::lambda(s)       ),

            // variants
            Box::new(|s| Token::sep(s)    ),
            Box::new(|s| Token::boolean(s)),

            // dynamic
            Box::new(|s| Token::real(s)  ),
            Box::new(|s| Token::string(s)),
            // Box::new(|s| Token::int(s)),

            // keep this @ the bottom, lmao
            Box::new(|s| Token::symbol(s) ),
        ];

        // maybe some sort of map reduce?
        let mut best = Consume::Err("Next token is not known in this context".to_string());

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

    fn literal(source: &str, literal: &str, kind: Token) -> Consume {
        Ok((kind, Token::expect(source, literal)?))
    }

    // token classifiers

    fn symbol(source: &str) -> Consume {
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

    fn open_bracket(source: &str) -> Consume {
        Token::literal(source, "{", Token::OpenBracket)
    }

    fn close_bracket(source: &str) -> Consume {
        Token::literal(source, "}", Token::CloseBracket)
    }

    fn open_paren(source: &str) -> Consume {
        Token::literal(source, "(", Token::OpenParen)
    }

    fn close_paren(source: &str) -> Consume {
        Token::literal(source, ")", Token::CloseParen)
    }

    fn assign(source: &str) -> Consume {
        Token::literal(source, "=", Token::Assign)
    }

    fn lambda(source: &str) -> Consume {
        Token::literal(source, "->", Token::Lambda)
    }

    fn real(source: &str) -> Consume {
        // TODO: NaNs, Infinity, the whole shebang
        // look at how f64::from_str is implemented, maybe?
        let mut len = 0;

        // one or more digits followed by a '.' followed by 1 or more digits
        len += Token::eat_digits(source)?;
        len += Token::expect(&source[len..], ".")?;
        len += Token::eat_digits(&source[len..])?;

        let number = match f64::from_str(&source[..len]) {
            Ok(n)  => n,
            Err(_) => panic!("Could not convert source to supposed real")
        };

        return Ok((Token::Number(Data::Real(number)), len));
    }

    // the below pattern is pretty common...
    // but I'm not going to abstract it out, yet

    fn string(source: &str) -> Consume {
        // TODO: read through the rust compiler and figure our how they do this
        // look into parse_str_lit

        let mut len    = 0;
        let mut escape = false;
        let mut string = "".to_string();

        len += Token::expect(source, "\"")?;

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

    fn boolean(source: &str) -> Consume {
        if let Ok(x) = Token::literal(source, "true", Token::Boolean(Data::Boolean(true))) {
            return Ok(x);
        }

        if let Ok(x) = Token::literal(source, "false", Token::Boolean(Data::Boolean(false))) {
            return Ok(x);
        }

        return Err("Expected a boolean".to_string());
    }

    fn sep(source: &str) -> Consume {
        match source.chars().next() {
            Some('\n') | Some(';') => Ok((Token::Sep, 1)),
            Some(_) => Err("Expected a separator, such as a newline".to_string()),
            None    => Err("Unexpected EOF while lexing".to_string()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnnToken<'a> {
    pub kind: Token,
    pub ann:  Ann<'a>,
}

impl AnnToken<'_> {
    pub fn new(kind: Token, ann: Ann) -> AnnToken {
        AnnToken { kind, ann }
    }
}

// cfg ain't working
#[cfg(test)]
mod test {
    use super::*;

    // each case tests the detection of a specific token type

    #[test]
    fn boolean() {
        assert_eq!(
            Token::from("true"),
            Ok((Token::Boolean(Data::Boolean(true)), 4)),
        );

        assert_eq!(
            Token::from("false"),
            Ok((Token::Boolean(Data::Boolean(false)), 5)),
        );
    }

    #[test]
    fn assign() {
        assert_eq!(
            Token::from("="),
            Ok((Token::Assign, 1)),
        );
    }

    #[test]
    fn symbol() {
        assert_eq!(
            Token::from("heck"),
            Ok((Token::Symbol(Local::new("heck".to_string())), 4))
        );
    }

    #[test]
    fn sep() {
        assert_eq!(
            Token::from("\nheck"),
            Ok((Token::Sep, 1)),
        );

        assert_eq!(
            Token::from("; heck"),
            Ok((Token::Sep, 1)),
        );
    }

    #[test]
    fn real() {
        assert_eq!(
            Token::from("2.0"),
            Ok((Token::Number(Data::Real(2.0)), 3)),
        );

        assert_eq!(
            Token::from("210938.2221"),
            Ok((Token::Number(Data::Real(210938.2221)), 11)),
        );
    }

    #[test]
    fn string() {
        let source = "\"heck\"";
        assert_eq!(
            Token::from(&source),
            Ok((Token::String(Data::String("heck".to_string())), source.len())),
        );

        let escape = "\"I said, \\\"Hello, world!\\\" didn't I?\"";
        assert_eq!(
            Token::from(&escape),
            Ok((
                Token::String(Data::String("I said, \"Hello, world!\" didn't I?".to_string())),
                escape.len()
            )),
        );

        // TODO: unicode support
    }
}
