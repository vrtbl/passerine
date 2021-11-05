use std::{
    iter::{once, Iterator},
    str::{FromStr, Chars},
    f64,
    rc::Rc,
};

use crate::common::{
    source::Source,
    span::{Span, Spanned},
    lit::Lit,
};

use crate::construct::{
    token::{Delim, Token, Tokens},
};

use crate::compiler::{
    syntax::{Syntax, Note},
};

// impl Lower for ThinModule<Rc<Source>> {
//     type Out = ThinModule<Tokens>;
//
//     /// Simple function that lexes a source file into a token stream.
//     /// Exposes the functionality of the `Lexer`.
//     fn lower(self) -> Result<Self::Out, Syntax> {
//         let mut lexer = Lexer::new(&self.repr);
//         return Ok(ThinModule::thin(lexer.all()?));
//     }
// }

pub struct Lexer {
    source:  Rc<Source>,
    index:   usize,
    nesting: Vec<usize>,
    tokens:  Tokens,
}

impl Lexer {
    pub fn lex(source: Rc<Source>) -> Result<Tokens, Syntax> {
        // build a base lexer for this file
        let mut lexer = Lexer {
            source,
            index: 0,
            nesting: vec![],
            tokens: vec![],
        };

        // prime the lexer
        lexer.strip();

        // consume!
        while lexer.index < lexer.source.contents.len() {
            // Insert the next token
            let token = lexer.next_token()?;
            lexer.tokens.push(token);

            // Strip whitespace, but not newlines, and comments
            lexer.strip();
        }

        // phew, nothing broke. Your tokens, sir!
        Ok(lexer.tokens)
    }

    fn grab_from_index(&self, len: usize) -> &str {
        &self.source.contents[self.index..self.index + len]
    }

    fn remaining(&self) -> Chars {
        self.source.contents[self.index..].chars()
    }

    fn strip(&mut self) {
        let remaining = self.remaining();

        loop {
            let old_index = self.index;

            // Strip whitespace
            while let Some(c) = remaining.next() {
                if !c.is_whitespace() || c == '\n' {
                    self.index += c.len_utf8();
                }
            }

            // TODO: doc comments and expression comments
            // Strip single line comment
            if let Some('-') = remaining.next() {
                if let Some('-') = remaining.next() {
                    // eat comment until the end of the line
                    while let Some(c) = remaining.next() {
                        if c != '\n' {
                            self.index += c.len_utf8();
                        }
                    }
                }
            }

            // If nothing was stripped, we're done
            if self.index == old_index { break }
        }
    }

    fn enter_group(&mut self, delim: Delim) -> (Token, usize) {
        self.nesting.push(self.index);
        (Token::Delim(delim, Rc::new(vec![])), 1)
    }

    fn exit_group(&mut self, delim: Delim) -> Result<Spanned<Token>, Syntax> {
        // get the location of the matching opening pair
        let loc = self.nesting.pop().ok_or(Syntax::error(
            &format!("Closing {} does not have an opening {}", delim, delim),
            &Span::point(&self.source, self.index),
        ))?;

        // split off new tokens, insert into group
        let after = self.tokens.split_off(loc + 1);
        let group = self.tokens.pop().unwrap();
        if let Token::Delim(delim, tokens) = group.item {
            *tokens = after;
        }

        // span over the whole group
        group.span = Span::combine(
            &group.span,
            &Span::point(&self.source, self.index)
        );

        Ok(group)
    }

    fn take_while<T>(
        &self,
        remaining: impl Iterator<Item = char>,
        wrap: impl Fn(&str) -> T,
        pred: impl Fn(char) -> bool,
    ) -> (T, usize) {
        let mut len = 0;
        while let Some(n) = remaining.next() {
            if !pred(n) { break; }
            len += n.len_utf8();
        }
        let inside = &self.grab_from_index(len);
        (wrap(inside), len)
    }

    fn string(&self, remaining: impl Iterator<Item = char>) -> Result<(Token, usize), Syntax> {
        // expects opening quote to have been parsed
        let mut len    = 1;
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
                                &format!("To include a single backslash `\\`, escape it first: `\\\\`"),
                                &Span::new(&self.source, self.index + len - bytes, 1 + bytes),
                            ),
                        )
                        // TODO: add help note about backslash escape
                    ),
                })
            } else {
                match c {
                    '\\' => escape = true,
                    '\"' => return Ok((Token::Lit(Lit::String(string)), len)),
                    c    => string.push(c),
                }
            }
        }

        Err(Syntax::error(
            "Unexpected end of source while parsing string literal",
            &Span::point(&self.source, self.index + len),
        ))
    }

    fn integer_literal(
        &self,
        radix: u32,
        remaining: impl Iterator<Item = char>,
    ) -> Result<(Token, usize), Syntax> {
        let (integer, len) = self.take_while(
            remaining,
            |s| i64::from_str_radix(s, radix)
                .map_err(|_| Syntax::error(
                    "Integer literal too large to fit in a signed 64-bit integer",
                    // hate the + 2 hack
                    // + 2 chars to take the `0?` into account
                    &Span::new(&self.source, self.index, s.len() + 2),
                )),
            |n| n.is_digit(radix),
        );
        Ok((Token::Lit(Lit::Integer(integer?)), len + 2))
    }

    fn radix_literal(
        &self,
        n: char,
        remaining: impl Iterator<Item = char>,
    ) -> Result<(Token, usize), Syntax> {
        match n {
            'b' => self.integer_literal(2, remaining),
            'o' => self.integer_literal(8, remaining), // Octal
            // Decimal, for kicks
            'd' => self.integer_literal(10, remaining),
            'x' => self.integer_literal(16, remaining),
            // Decimal literal with a leading zero
            _   => self.decimal_literal(
                // rebuild the iterator, ugh
                once('0').chain(once(n)).chain(remaining)
            ),
        }
    }

    fn decimal_literal(
        &self,
        remaining: impl Iterator<Item = char>,
    ) -> Result<(Token, usize), Syntax> {
        let mut len = self.take_while(
            remaining,
            |_| (),
            |n| n.is_digit(10),
        ).1;

        match remaining.next() {
            // There's a decimal point, so we parse as a float
            Some('.') => {
                len += self.take_while(
                    remaining,
                    |_| (),
                    |n| n.is_digit(10),
                ).1;
                let float = f64::from_str(&self.grab_from_index(len))
                    .map_err(|_| Syntax::error(
                        "Float literal does not fit in a 64-bit floating-point number",
                        &Span::new(&self.source, self.index, len),
                    ))?;
                Ok((Token::Lit(Lit::Float(float)), len))
            },
            // There's an 'E', so we parse using scientific notation
            Some('E') => {
                Err(Syntax::error(
                    "Scientific notation for floating-point is WIP!",
                    &Span::point(&self.source, self.index),
                ))
            },
            // Nothing of use, wrap up what we have so far
            _ => {
                let integer = i64::from_str(&self.grab_from_index(len))
                    .map_err(|_| Syntax::error(
                        "Decimal literal too large to fit in a signed 64-bit integer",
                        &Span::new(&self.source, self.index, len),
                    ))?;
                Ok((Token::Lit(Lit::Integer(integer)), len))
            }
        }
    }

    /// Parses the next token.
    /// Expects all whitespace and comments to be stripped.
    fn next_token(&mut self) -> Result<Spanned<Token>, Syntax> {
        let remaining = self.remaining();

        let (token, len) = match remaining.next().unwrap() {
            // separator
            c @ ('\n' | ';') => self.take_while(
                once(c).chain(remaining),
                |_| Token::Sep,
                |n| n.is_whitespace() || n == ';'
            ),

            // the unit type, `()`
            '(' if Some(')') == remaining.next() => {
                (Token::Lit(Lit::Unit), 2)
            },

            // Grouping
            '(' => self.enter_group(Delim::Paren),
            '{' => self.enter_group(Delim::Curly),
            '[' => self.enter_group(Delim::Square),
            ')' => return self.exit_group(Delim::Paren),
            '}' => return self.exit_group(Delim::Curly),
            ']' => return self.exit_group(Delim::Square),

            // Label
            c if c.is_alphabetic() && c.is_uppercase() => {
                self.take_while(
                    once(c).chain(remaining),
                    |s| match s {
                        // TODO: In the future, booleans in prelude
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
                    once(c).chain(remaining),
                    |s| Token::Iden(s.to_string()),
                    |n| n.is_alphanumeric() || n == '_'
                )
            },

            // Op
            c if c.is_ascii_punctuation() => {
                self.take_while(
                    once(c).chain(remaining),
                    |s| Token::Label(s.to_string()),
                    |n| n.is_ascii_punctuation(),
                )
            },

            // Number literal:
            // Integer: 28173908, etc.
            // Radix:   0b1011001011, 0xFF, etc.
            // Float:   420.69, 0., etc.
            c @ '0'..='9' => {
                if c != '0' {
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
                    self.decimal_literal(remaining)?
                }
            }

            // String
            '"' => self.string(remaining)?,

            // Unrecognized char
            unknown => return Err(Syntax::error(
                &format!(
                    "Hmm... The character `{}` is not recognized - check for encoding issues or typos",
                    unknown,
                ),
                &Span::point(&self.source, self.index),
            )),
        };

        let spanned = Spanned::new(
            token,
            Span::new(&self.source, self.index, len)
        );

        self.index += len;
        Ok(spanned)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::lit::Lit;

    // NOTE: lexing individual tokens is tested in pipeline::token

    #[test]
    fn new_empty() {
        Lexer::lex(Source::source("")).unwrap();
    }
}
