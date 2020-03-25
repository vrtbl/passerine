#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Token {
    // Whitespace
    Sep,

    // Lambda
    Assign,

    // Datatypes
    Symbol,
    Boolean,
}

type Consume = Option<(Token, usize)>;

// TODO: make moar unicode friendly
// let rust's stdlib deal with it

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
            Box::new(|s| Token::sep(s)    ),
            Box::new(|s| Token::assign(s) ),
            Box::new(|s| Token::boolean(s)),

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

    fn literal(source: &str, literal: &str, kind: Token) -> Consume {
        if literal.len() > source.len() { return None }

        if &source[..literal.len()] == literal {
            return Some((kind, literal.len()));
        }

        return None;
    }

    fn symbol(source: &str) -> Consume {
        // for now, a symbol is one or more ascii alphanumerics
        let mut len = 0;

        for char in source.chars() {
            if !char.is_ascii_alphanumeric() {
                break;
            }
            len += 1;
        }

        return match len {
            0 => None,
            l => Some((Token::Symbol, l)),
        };
    }

    fn assign(source: &str) -> Consume {
        return Token::literal(source, "=", Token::Assign);
    }

    // the below pattern is pretty common...
    // but I'm not going to abstract it out, yet

    fn boolean(source: &str) -> Consume {
        // possible duplication of knowledge, see parser.
        match Token::literal(source, "true",  Token::Boolean) {
            Some(x) => return Some(x),
            None => ()
        }

        match Token::literal(source, "false", Token::Boolean) {
            Some(x) => return Some(x),
            None => return None
        }
    }

    fn sep(source: &str) -> Consume {
        match source.chars().next()? {
            '\n' | ';' => Some((Token::Sep, 1)),
            _          => None
        }
    }
}

// cfg ain't working
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn boolean() {
        assert_eq!(
            Token::from("true"),
            Some((Token::Boolean, 4)),
        );

        assert_eq!(
            Token::from("false"),
            Some((Token::Boolean, 5)),
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
            Some((Token::Symbol, 4))
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
}
