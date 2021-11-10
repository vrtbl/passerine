use crate::common::data::Data;
use std::fmt::Display;

// TODO: remove associated data from tokens.

/// These are the different tokens the lexer will output.
/// `Token`s with data contain that data,
/// e.g. a boolean will be a `Data::Boolean(...)`, not just a string.
/// `Token`s can be spanned using `Spanned<Token>`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    // Delimiters
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    Sep,
    Pair,

    // Keywords
    Syntax,
    Assign,
    Lambda,
    Compose,
    Magic,
    // pseudokeywords
    Keyword(String),

    // Datatypes
    // TODO: just have one variant, `Data`
    Unit,
    Number(Data),
    String(Data),
    Boolean(Data),

    // defined by span rather than be contents
    Symbol,
    Label,

    // Operators
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,

    Equal,

    // EoS
    End,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // pretty formatting for tokens
        // just use debug if you're not printing a message or something.
        let message = match self {
            Token::OpenBracket => "an opening bracket",
            Token::CloseBracket => "a closing bracket",
            Token::OpenParen => "an openening paren",
            Token::CloseParen => "a closing paren",
            Token::Sep => "a separator",
            Token::Syntax => "a syntax definition",
            Token::Assign => "an assignment",
            Token::Lambda => "a lambda",
            Token::Compose => "a composition",
            Token::Unit => "the Unit, '()'",
            Token::Pair => "a tuple",
            Token::Magic => "a magic keyword",
            Token::Symbol => "a symbol",
            Token::Label => "a Label", // capitilized to mimic actual labels
            Token::Number(_) => "a number",
            Token::String(_) => "a string",
            Token::Add => "an addition",
            Token::Sub => "a subtraction",
            Token::Mul => "a multiplication",
            Token::Div => "a division",
            Token::Rem => "a remainder",
            Token::Pow => "a power of",
            Token::Equal => "an equality test",
            Token::End => "end of source",
            Token::Keyword(k) => {
                return write!(f, "the pseudokeyword '{}", k);
            }
            Token::Boolean(b) => {
                return write!(f, "the boolean {}", b);
            }
        };
        write!(f, "{}", message)
    }
}
