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
}
