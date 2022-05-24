use std::{
    f64,
    iter::{once, Iterator, Peekable},
    rc::Rc,
    str::{Chars, FromStr},
};

use crate::{
    common::{
        lit::Lit,
        source::Source,
        span::{Span, Spanned},
    },
    compiler::syntax::{Note, Syntax},
    construct::token::{Delim, Token, Tokens},
};

const OP_CHARS: &str = "!$%&*+,-./:<=>?@^|~";

macro_rules! RemainingIter {
    () => { Peekable<impl Iterator<Item = char>> };
}

#[derive(Debug)]
pub struct Lexer {
    source: Rc<Source>,
    index: usize,
    tokens: Tokens,
}

impl Lexer {
    // TODO: lexer needs to return all macro declarations
    /// Lexes a source file into a stream of tokens.
    pub fn lex(source: Rc<Source>) -> Result<Spanned<Tokens>, Syntax> {
        // get a span that spans the entire source file:
        let span = Span::new(&source, 0, source.contents.len());

        // build a base lexer for this file
        let mut lexer = Lexer {
            source,
            index: 0,
            tokens: vec![],
        };

        // prime the lexer
        lexer.strip();

        // consume all!
        while lexer.index < lexer.source.contents.len() {
            let token = lexer.next_token()?;
            lexer.tokens.push(token);
            lexer.strip();
        }

        // phew, nothing broke. Your tokens, sir!
        Ok(Spanned::new(lexer.tokens, span))
    }

    /// Selects a range of a string of length `len` from the
    /// current index position.
    fn grab_from_index(&self, len: usize) -> &str {
        &self.source.contents[self.index..self.index + len]
    }

    /// Returns all characters after the current index
    /// position.
    fn remaining(&self) -> Chars {
        self.source.contents[self.index..].chars()
    }

    // TODO: use own index instead of self.index
    fn strip(&mut self) {
        loop {
            let mut remaining = self.remaining().peekable();
            let mut new_index = self.index;
            let old_index = new_index;

            // strip whitespace...
            while let Some(c) = remaining.peek() {
                // ...but don't strip newlines!
                if !c.is_whitespace() || *c == '\n' {
                    break;
                }
                new_index += c.len_utf8();
                remaining.next();
            }

            // Strip single line comment
            if let Some('#') = remaining.next() {
                // the comment `#` length
                new_index += 1;
                // eat comment until the end of the line
                for c in remaining {
                    if c == '\n' {
                        break;
                    }
                    new_index += c.len_utf8();
                }
            }

            // If nothing was stripped, we're done
            self.index = new_index;
            if old_index == new_index {
                break;
            }
        }
    }

    /// Starting at the parser's current index.
    /// consumes characters one at a time according to a
    /// `pred`icate. after the predicate returns false,
    /// the string is passed to a `wrap` function, which
    /// converts the string slice of consumed characters
    /// into a type `T`, and returns that type along
    /// with the number of bytes consumed. (The number
    /// of bytes consumed can be used to advance
    /// `self.index`.)
    fn take_while<T>(
        &self,
        remaining: &mut RemainingIter!(),
        wrap: impl Fn(&str) -> T,
        pred: impl Fn(char) -> bool,
    ) -> (T, usize) {
        let mut len = 0;
        while let Some(n) = remaining.peek() {
            if !pred(*n) {
                break;
            }
            len += n.len_utf8();
            remaining.next();
        }
        let inside = &self.grab_from_index(len);
        (wrap(inside), len)
    }

    fn string(
        &self,
        remaining: RemainingIter!(),
    ) -> Result<(Token, usize), Syntax> {
        // expects opening quote to have been parsed
        let mut len = 1;
        let mut escape = false;
        let mut string = String::new();

        for c in remaining {
            let bytes = c.len_utf8();
            len += bytes;
            if escape {
                escape = false;
                // TODO: nesting expression inside strings for splicing
                // TODO: \x and \u{..} for ascii and unicode
                // TODO: maybe add parsing escape codes to later step?
                string.push(match c {
                    '"'  => '"',
                    '\\' => '\\',
                    'n'  => '\n',
                    'r'  => '\r',
                    't'  => '\t',
                    '0'  => '\0',
                    o    => return Err(
                        Syntax::error_with_note(
                            &format!("Unknown escape code `\\{}` in string literal", o),
                            Note::new_with_hint(
                                "To include a single backslash `\\`, escape it first: `\\\\`",
                                &Span::new(&self.source, self.index + len - bytes, bytes),
                            ),
                        )
                        // TODO: add help note about backslash escape
                    ),
                })
            } else {
                match c {
                    '\\' => escape = true,
                    '\"' => return Ok((Token::Lit(Lit::String(string)), len)),
                    c => string.push(c),
                }
            }
        }

        Err(Syntax::error(
            "Unexpected end of source while parsing string literal",
            &Span::point(&self.source, self.index + len),
        ))
    }

    /// Must start with two-byte prefix `0?`, where `?`
    /// indicates radix.
    fn integer_literal(
        &self,
        radix: u32,
        mut remaining: RemainingIter!(),
    ) -> Result<(Token, usize), Syntax> {
        let len = 2 + self
            .take_while(&mut remaining, |_| (), |n| n.is_digit(radix))
            .1;

        let integer =
            i64::from_str_radix(&self.grab_from_index(len)[2..], radix)
                .map_err(|_| {
                    Syntax::error(
                "Integer literal too large to fit in a signed 64-bit integer",
                // hate the + 2 hack
                // + 2 chars to take the `0?` into account
                &Span::new(&self.source, self.index, len),
            )
                });

        Ok((Token::Lit(Lit::Integer(integer?)), len))
    }

    fn radix_literal(
        &self,
        n: char,
        // remaining does not lead with `n`
        remaining: RemainingIter!(),
    ) -> Result<(Token, usize), Syntax> {
        // TODO: figure out something more elegant than this += 2 -=
        // 2 ordeal
        match n {
            'b' => self.integer_literal(2, remaining),
            'o' => self.integer_literal(8, remaining), // Octal
            // Decimal, for kicks
            'd' => self.integer_literal(10, remaining),
            'x' => self.integer_literal(16, remaining),
            // Decimal literal with a leading zero
            _ => {
                // rebuild the iterator, ugh
                let remaining = once('0').chain(once(n)).chain(remaining);
                self.decimal_literal(remaining.peekable())
            },
        }
    }

    fn decimal_literal(
        &self,
        mut remaining: RemainingIter!(),
    ) -> Result<(Token, usize), Syntax> {
        let mut len = self
            .take_while(&mut remaining, |_| (), |n| n.is_digit(10))
            .1;

        match remaining.next() {
            // There's a decimal point, so we parse as a float
            Some('.') => {
                len += 1; // for the '.'
                len += self
                    .take_while(&mut remaining, |_| (), |n| n.is_digit(10))
                    .1;
                let float = f64::from_str(self.grab_from_index(len))
                    .map_err(|_| Syntax::error(
                        "Float literal does not fit in a 64-bit floating-point number",
                        &Span::new(&self.source, self.index, len),
                    ))?;
                Ok((Token::Lit(Lit::Float(float)), len))
            },
            // There's an 'e', so we parse using scientific notation
            Some('e') => Err(Syntax::error(
                "Scientific notation for floating-point is WIP!",
                &Span::point(&self.source, self.index),
            )),
            // Nothing of use, wrap up what we have so far
            _ => {
                let integer = i64::from_str(self.grab_from_index(len))
                    .map_err(|_| Syntax::error(
                        "Decimal literal too large to fit in a signed 64-bit integer",
                        &Span::new(&self.source, self.index, len),
                    ))?;
                Ok((Token::Lit(Lit::Integer(integer)), len))
            },
        }
    }

    /// Parses the next token.
    /// Expects all whitespace and comments to be stripped.
    fn next_token(&mut self) -> Result<Spanned<Token>, Syntax> {
        let mut remaining = self.remaining().peekable();

        let (token, len) = match remaining.next().unwrap() {
            // separator
            c @ ('\n' | ';') => self.take_while(
                &mut once(c).chain(remaining).peekable(),
                |_| Token::Sep,
                |n| n.is_whitespace() || n == ';'
            ),

            // the unit type, `()`
            '(' if Some(')') == remaining.next() => {
                (Token::Lit(Lit::Unit), 2)
            },

            // Grouping
            '(' => (Token::Open(Delim::Paren), 1),
            '{' => (Token::Open(Delim::Curly), 1),
            '[' => (Token::Open(Delim::Square), 1),
            ')' => (Token::Close(Delim::Paren), 1),
            '}' => (Token::Close(Delim::Curly), 1),
            ']' => (Token::Close(Delim::Square), 1),

            // Label
            c if c.is_alphabetic() && c.is_uppercase() => {
                self.take_while(
                    &mut once(c).chain(remaining).peekable(),
                    |s| match s {
                        // TODO: In the future, booleans in prelude as ADTs
                        "True" => Token::Lit(Lit::Boolean(true)),
                        "False" => Token::Lit(Lit::Boolean(false)),
                        _ => Token::Label(s.to_string()),
                    },
                    |n| n.is_alphanumeric() || n == '_'
                )
            },

            // Iden
            c if c.is_alphabetic() || c == '_' => {
                self.take_while(
                    &mut once(c).chain(remaining).peekable(),
                    |s| Token::Iden(s.to_string()),
                    |n| n.is_alphanumeric() || n == '_'
                )
            },

            // Number literal:
            // Integer: 28173908, etc.
            // Radix:   0b1011001011, 0xFF, etc.
            // Float:   420.69, 0.0, etc.
            c @ '0'..='9' => {
                if c == '0' {
                    if let Some(n) = remaining.next() {
                        // Potentially integers in other radixes
                        self.radix_literal(n, remaining)?
                    } else {
                        // End of source, must be just `0`
                        (Token::Lit(Lit::Integer(0)), 1)
                    }
                } else {
                    // parse decimal literal
                    // this could be an integer
                    // but also a floating point number
                    self.decimal_literal(once(c).chain(remaining).peekable())?
                }
            }

            // String
            '"' => self.string(remaining)?,

            // TODO: choose characters for operator set
            // don't have both a list and `is_ascii_punctuation`
            // Op
            c if OP_CHARS.contains(c) => {
                self.take_while(
                    &mut once(c).chain(remaining).peekable(),
                    |s| Token::Op(s.to_string()),
                    |n| OP_CHARS.contains(n),
                )
            },

            // Unrecognized char
            unknown => return Err(Syntax::error(
                &format!(
                    "Hmm... The character `{}` is not recognized in this context - check for encoding issues or typos",
                    unknown,
                ),
                &Span::point(&self.source, self.index),
            )),
        };

        let spanned =
            Spanned::new(token, Span::new(&self.source, self.index, len));

        self.index += len;
        Ok(spanned)
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use super::*;
    use crate::common::lit::Lit;

    // NOTE: lexing individual tokens is tested in
    // pipeline::token

    proptest! {
        #[test]
        fn doesnt_crash(s in "\\PC*") {
            let result = Lexer::lex(Source::source(&s));
            format!("{:?}", result);
        }

        #[test]
        fn integers(s in "-?[0-9]+") {
            let result = Lexer::lex(Source::source(&s));
            format!("{:?}", result);
        }

        #[test]
        fn operators(s in "[!$%&*+,-./:<=>?@^|~]+") {
            let result = Lexer::lex(Source::source(&s));
            prop_assert!(result.is_ok());
            if let Token::Op(op) = &result.unwrap().item[0].item {
                prop_assert_eq!(op, &s);
            }
        }

        #[test]
        fn small_positive_floats(x in 0.0..1000000.0) {
            let formatted = format!("{:?}", x);
            dbg!(&formatted);
            let result = Lexer::lex(Source::source(&formatted));
            dbg!(&result);
            prop_assert!(result.is_ok());
            let unwrapped = result.unwrap().item;
            prop_assert!(unwrapped.len() == 1);
            prop_assert_eq!(&unwrapped[0].item, &Token::Lit(Lit::Float(x)));
        }
    }

    #[test]
    fn decimal_float() {
        let x = 38328388.30363078;
        let formatted = format!("{:?}", x);
        let result = Lexer::lex(Source::source(&formatted));
        assert!(result.is_ok());
        let unwrapped = result.unwrap().item;
        assert!(unwrapped.len() == 1);
        assert_eq!(unwrapped[0].item, Token::Lit(Lit::Float(x)));
    }

    #[test]
    fn zero_float() {
        let result = Lexer::lex(Source::source("0.0"));
        assert_eq!(result.unwrap().item[0].item, Token::Lit(Lit::Float(0.0)));
    }

    #[test]
    fn brackets() {
        let result = Lexer::lex(Source::source("{[(])}()")).unwrap().item;
        assert_eq!(result[0].item, Token::Open(Delim::Curly));
        assert_eq!(result[1].item, Token::Open(Delim::Square));
        assert_eq!(result[2].item, Token::Open(Delim::Paren));
        assert_eq!(result[3].item, Token::Close(Delim::Square));
        assert_eq!(result[4].item, Token::Close(Delim::Paren));
        assert_eq!(result[5].item, Token::Close(Delim::Curly));
        assert_eq!(result[6].item, Token::Lit(Lit::Unit));
    }

    #[test]
    fn unclosed_string() {
        let result = Lexer::lex(Source::source("\"asdf\"\"qwerty"));
        assert!(result.is_err());
    }

    #[test]
    fn escape_code() {
        let s = "\"\\᪠\u{16f4f}";
        let result = Lexer::lex(Source::source(&s));
        format!("{:?}", result);
    }

    #[test]
    fn new_empty() {
        Lexer::lex(Source::source("")).unwrap();
    }
}
