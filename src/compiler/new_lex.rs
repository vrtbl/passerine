use std::{
    iter::Peekable,
    str::{FromStr, Chars},
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

use crate::compiler::{
    lower::Lower,
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
    tokens:  Vec<Spanned<Token>>,
}

impl Lexer {
    pub fn lex(source: Rc<Source>) -> Result<Tokens, Syntax> {
        let mut lexer = Lexer {
            source,
            index: 0,
            nesting: vec![],
            tokens:  vec![],
        };

        todo!();
    }

    fn enter_group(&mut self, delim: Delim) -> (Token, usize) {
        self.nesting.push(self.index);
        (Token::empty_group(delim), 1)
    }

    fn exit_group(&mut self, delim: Delim) -> Result<Spanned<Token>, Syntax> {
        // get the location of the matching opening pair
        let loc = self.nesting.pop().ok_or(Syntax::error(
            "Closing parenthesis `)` without corresponding opening parenthesis `(`",
            &Span::point(&self.source, self.index),
        ))?;

        // split off new tokens, insert into group
        let after = self.tokens.split_off(loc + 1);
        let group = self.tokens.pop().unwrap();
        if let Token::Group { delim, ref mut tokens } = group.item {
            *tokens = after;
        }

        // span over the whole group
        group.span = Span::combine(
            &group.span,
            &Span::point(&self.source, self.index)
        );

        Ok(group)
    }

    fn literal_while(
        &self,
        c: char,
        remaining: Chars,
        wrap: impl Fn(String) -> Token,
        pred: impl Fn(char) -> bool,
    ) -> (Token, usize) {
        let mut used = c.len_utf8();
        while let Some(n) = remaining.next() {
            if !pred(n) { break; }
            used += n.len_utf8();
        }
        let inside = &self.source.contents[self.index..self.index + used];
        (wrap(inside.to_string()), used)
    }

    fn string(&self, remaining: Chars) -> Result<(Token, usize), Syntax> {
        // expects opening quote to have been parsed
        let mut len    = 1;
        let mut escape = false;
        let mut string = String::new();

        for c in remaining {
            len += c.len_utf8();
            if escape {
                escape = false;
                // TODO: add more escape codes
                // TODO: maybe add parsing escape codes to later step?
                string.push(match c {
                    '"'  => '"',
                    '\\' => '\\',
                    'n'  => '\n',
                    't'  => '\t',
                    'r'  => '\r',
                    o    => return Err(
                        Syntax::error(
                            &format!("Unknown escape code `\\{}` in string literal", o),
                            &Span::new(&self.source, self.index + len - 1, o.len_utf8()),
                        )
                        // TODO: add help note about backslash escape
                    ),
                })
            } else {
                match c {
                    '\\' => escape = true,
                    '\"' => return Ok((Token::Data(Data::String(string)), len)),
                    c    => string.push(c),
                }
            }
        }

        Err(Syntax::error(
            "Unexpected end of source while parsing string literal",
            &Span::point(&self.source, self.index + len),
        ))
    }

    /// Parses the next token.
    /// Expects all whitespace and comments to be stripped.
    fn next_token(&mut self) -> Result<Spanned<Token>, Syntax> {
        let remaining = self.source.contents[self.index..].chars();

        let (token, used) = match remaining.next().unwrap() {
            // the unit type, `()`
            '(' if Some(')') == remaining.next() => {
                (Token::Data(Data::Unit), 2)
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
                self.literal_while(c, remaining, Token::Label, |n| {
                    n.is_alphanumeric() || n == '_'
                })
            },
            // Iden
            c if c.is_alphabetic() || c == '_' => {
                self.literal_while(c, remaining, Token::Iden, |n| {
                    n.is_alphanumeric() || n == '_'
                })
            },
            // Op
            c if c.is_ascii_punctuation() => {
                self.literal_while(c, remaining, Token::Op, |n| {
                    n.is_ascii_punctuation()
                })
            },

            // Number
            c @ '0'..='9' => {
                if let Some(n) = remaining.peek() {
                    // handle other bases
                    match n {
                        'b' => todo!(),
                        'c' => todo!(), // Octal
                        'x' => todo!(),
                    }
                } else {
                    // decimal base
                    todo!()
                    // self.decimal(c, remaining)
                }
            },

            // String
            '"' => self.string(remaining),

            // Unrecognized char
            _ => todo!(),
        };

        let spanned = Spanned::new(
            token,
            Span::new(&self.source, self.index, used)
        );

        self.index += used;
        Ok(spanned)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::data::Data;

    // NOTE: lexing individual tokens is tested in pipeline::token

    #[test]
    fn new_empty() {
        lex(&Source::source("")).unwrap();
    }
}
