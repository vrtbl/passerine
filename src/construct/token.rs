use std::fmt::Display;
use crate::common::{
    span::Spanned,
    data::Data,
};

// TODO: remove associated data from tokens.

/// These are the different tokens the lexer will output.
/// `Token`s with data contain that data,
/// e.g. a boolean will be a `Data::Boolean(...)`, not just a string.
/// `Token`s can be spanned using `Spanned<Token>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Delimiters
    Group {
        delim: Delim,
        tokens: Tokens,
    },

    // Names
    Label(String),
    Iden(String),
    Op(String),

    // Values
    Data(Data),

    // Context
    Sep,
    End,
}

pub type Tokens = Vec<Spanned<Token>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delim {
    Paren,
    Curly,
    Square,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResIden {
    Type,
    Magic,
}

impl ResIden {
    pub fn try_new(name: &str) -> Option<ResIden> {
        use ResIden::*;
        Some(match name {
            "type"  => Type,
            "magic" => Magic,
            _ => { return None; },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResOp {
    Assign,
    Lambda,
    Equal,
    Pow,
    Compose,
    Pair,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

impl ResOp {
    pub fn try_new(name: &str) -> Option<ResOp> {
        use ResOp::*;
        Some(match name {
            "=" => Assign,
            "->" => Lambda,
            "==" => Equal,
            "**" => Pow,
            "." => Compose,
            "," => Pair,
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,
            "%" => Rem,
            _ => { return None; },
        })
    }
}

impl Display for Delim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Delim::Paren => "parenthesis",
            Delim::Curly => "curly brackets",
            Delim::Square => "square brackets",
        };

        write!(f, "{}", message)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // pretty formatting for tokens
        // just use debug if you're not printing a message or something.
        let message = match self {
            Token::Group { delim, .. } => format!("tokens grouped by {}", delim),
            Token::Label(l)            => format!("the label `{}`", l),
            Token::Iden(i)             => format!("the identifier `{}`", i),
            Token::Op(o)               => format!("the operator `{}`", o),
            Token::Data(d)             => format!("the data `{}`", d),
            Token::Sep                 => "a separator".to_string(),
            Token::End                 => "the end of source".to_string(),
        };

        write!(f, "{}", message)
    }
}
