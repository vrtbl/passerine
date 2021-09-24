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

    /// Parses the next token.
    /// Expects all whitespace and comments to be stripped.
    fn next_token(&mut self) -> Result<Spanned<Token>, Syntax> {
        let remaining = self.source.contents[self.index..].chars();

        let (token, used) = match remaining.next().unwrap() {
            // the unit type, `()`
            '(' if Some(')') == remaining.next() => {
                (Token::Data(Data::Unit), 2)
            },
            '(' => self.enter_group(Delim::Paren),
            '{' => self.enter_group(Delim::Curly),
            '[' => self.enter_group(Delim::Square),
            ')' => return self.exit_group(Delim::Paren),
            '}' => return self.exit_group(Delim::Curly),
            ']' => return self.exit_group(Delim::Square),
        };

        let spanned = Spanned::new(
            token,
            Span::new(&self.source, self.index, used)
        );

        self.index += used;
        Ok(spanned)
    }
}

pub fn lex(source: &Rc<Source>) -> Result<Tokens, Syntax> {
    let contents = &source.contents;
    let mut tokens: Tokens = vec![];
    let mut nesting = vec![];
    let mut index = 0;

    while index < contents.len() {
        let mut remaining = &mut contents[index..].chars();

        let (token, used) = if let Some(c) = remaining.next() {
            match c {
                '(' => if let Some(')') = remaining.next() {
                    (Token::Data(Data::Unit), 2)
                } else {
                    nesting.push(index);
                    (Token::empty_group(Delim::Paren), 1)
                }
                '{' => {
                    nesting.push(index);
                    (Token::empty_group(Delim::Curly), 1)
                }
                '[' => {
                    nesting.push(index);
                    (Token::empty_group(Delim::Square), 1)
                }
                ')' => {
                    let loc = nesting.pop().ok_or(Syntax::error(
                        "Closing parenthesis `)` without corresponding opening parenthesis `(`",
                        &Span::point(source, index),
                    ))?;
                    let after = tokens.split_off(loc + 1);
                    let group = tokens.pop().unwrap();
                    if let Token::Group { delim, ref mut tokens } = group.item {
                        *tokens = after;
                    }
                    group.span = Span::combine(&group.span, &Span::point(source, index));
                    // TODO: span over whole group
                    (group.item, 1)
                }
                _ => todo!(),
            }
        } else {
            return Err(
                Syntax::error(
                    "File ended in unexpected way, try removing characters after this point.",
                    &Span::new(source, index, contents.len() - index),
                )
            );
        };

        let spanned = Spanned::new(token, Span::new(source, index, used));
        index += used;
        tokens.push(spanned);
    }

    return Ok(tokens);
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
