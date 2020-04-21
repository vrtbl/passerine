use std::str::FromStr;
use std::f64;

use crate::utils::annotation::Ann;
use crate::vm::data::Data;
use crate::vm::local::Local;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    // Delimiterss
    OpenBracket,
    CloseBracket,
    Sep,

    // Lambda
    Assign,

    // Datatypes
    Symbol(Local),
    Number(Data),
    Boolean(Data),
}

type Consume = Option<(Token, usize)>;

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
            Box::new(|s| Token::assign(s)       ),

            // variants
            Box::new(|s| Token::sep(s)    ),
            Box::new(|s| Token::boolean(s)),

            // dynamic
            Box::new(|s| Token::real(s)),
            // Box::new(|s| Token::int(s)),

            // keep this @ the bottom, lmao
            Box::new(|s| Token::symbol(s) ),
        ];

        // maybe some sort of map reduce?
        let mut best = None;

        // check longest
        for rule in &rules {
            if let Some((k, c)) = rule(source) {
                match best {
                    None                  => best = Some((k, c)),
                    Some((_, o)) if c > o => best = Some((k, c)),
                    Some(_)               => (),
                }
            }
        }

        return best;
    }

    // helpers

    fn literal(source: &str, literal: &str, kind: Token) -> Consume {
        if literal.len() > source.len() { return None }

        if &source[..literal.len()] == literal {
            return Some((kind, literal.len()));
        }

        return None;
    }

    fn eat_digits(source: &str) -> Option<usize> {
        let mut len = 0;

        for char in source.chars() {
            match char {
                n if n.is_digit(10) => len += 1,
                _                   => break,
            }
        }

        return if len == 0 { None } else { Some(len) };
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
            0 => None,
            // TODO: make sure that symbol name is correct
            l => Some((Token::Symbol(Local::new(source[..l].to_string())), l)),
        };
    }

    fn open_bracket(source: &str) -> Consume {
        return Token::literal(source, "{", Token::OpenBracket);
    }

    fn close_bracket(source: &str) -> Consume {
        return Token::literal(source, "}", Token::CloseBracket);
    }

    // NEXT: parse

    fn assign(source: &str) -> Consume {
        return Token::literal(source, "=", Token::Assign);
    }

    fn real(source: &str) -> Consume {
        // TODO: NaNs, Infinity, the whole shebang
        // look at how f64::from_str is implemented, maybe?

        let mut len: usize = 0;

        // one or more digits followed by a '.' followed by 1 or more digits
        // TODO: make more general
        len += Token::eat_digits(source)?;

        match source[len..].chars().next() {
            Some('.') => len += 1,
            _         => return None,
        }

        len += Token::eat_digits(&source[len..])?;

        let number = match f64::from_str(&source[..len]) {
            Ok(n)  => n,
            Err(_) => panic!("Could not convert source to supposed real")
        };

        return Some((Token::Number(Data::Real(number)), len));
    }

    // the below pattern is pretty common...
    // but I'm not going to abstract it out, yet

    fn boolean(source: &str) -> Consume {
        if let Some(x) = Token::literal(source, "true", Token::Boolean(Data::Boolean(true))) {
            return Some(x);
        }

        if let Some(x) = Token::literal(source, "false", Token::Boolean(Data::Boolean(false))) {
            return Some(x);
        }

        return None;
    }

    fn sep(source: &str) -> Consume {
        match source.chars().next()? {
            '\n' | ';' => Some((Token::Sep, 1)),
            _          => None
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnnToken {
    pub kind: Token,
    pub ann:  Ann,
}

impl AnnToken {
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
            Some((Token::Boolean(Data::Boolean(true)), 4)),
        );

        assert_eq!(
            Token::from("false"),
            Some((Token::Boolean(Data::Boolean(false)), 5)),
        );
    }

    #[test]
    fn assign() {
        assert_eq!(
            Token::from("="),
            Some((Token::Assign, 1)),
        );
    }

    #[test]
    fn symbol() {
        assert_eq!(
            Token::from(""),
            None,
        );

        assert_eq!(
            Token::from("heck"),
            Some((Token::Symbol(Local::new("heck".to_string())), 4))
        );
    }

    #[test]
    fn sep() {
        assert_eq!(
            Token::from("\nheck"),
            Some((Token::Sep, 1)),
        );

        assert_eq!(
            Token::from("; heck"),
            Some((Token::Sep, 1)),
        );
    }

    #[test]
    fn real() {
        assert_eq!(
            Token::from("2.0"),
            Some((Token::Number(Data::Real(2.0)), 3)),
        );

        assert_eq!(
            Token::from("210938.2221"),
            Some((Token::Number(Data::Real(210938.2221)), 11)),
        );
    }
}
