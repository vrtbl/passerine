use std::fmt::Display;

use crate::common::{
    span::Span,
    data::Data,
};

/// These are the different tokens the lexer will output.
/// `Token`s with data contain that data,
/// e.g. a boolean will be a Data::Boolean(...), not just a string.
/// `Token`s can be spanned using `Spanned<Token>`.
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
    Symbol, // is specified by Span rather than an actual value
    Number(Data),
    String(Data),
    Boolean(Data),

    // EoS
    End
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Token::OpenBracket  => "an opening bracket",
            Token::CloseBracket => "a closing bracket",
            Token::OpenParen    => "an openening paren",
            Token::CloseParen   => "a closing paren",
            Token::Sep          => "a separator",
            Token::Assign       => "an assignment",
            Token::Lambda       => "a lambda",
            Token::Symbol       => "a symbol",
            Token::Number(_)    => "a number",
            Token::String(_)    => "a string",
            Token::Boolean(_)   => "a boolean, like 'True' or 'False'",
            Token::End          => "the end of source"
        };
        write!(f, "{}", message)
    }
}
