use std::fmt::Display;

use crate::common::{lit::Lit, span::Spanned};

pub type Tokens = Vec<Spanned<Token>>;

/// These are the different tokens the lexer will output.
/// `Token`s with data contain that data,
/// e.g. a boolean will be a `Lit::Boolean(...)`, not just a string.
/// `Token`s can be spanned using `Spanned<Token>`.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Grouping
    Block(Vec<Tokens>),
    List(Tokens),
    Form(Tokens),

    // Leafs
    Iden(String),
    Op(String),
    Lit(Lit),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // pretty formatting for tokens
        // just use debug if you're not printing a message or something.
        let message = match self {
            Token::Block(_) => format!("tokens grouped by curly brackets"),
            Token::List(_) => format!("tokens grouped by square brackets"),
            Token::Form(_) => format!("a group of tokens"),
            Token::Iden(i) => format!("identifier `{}`", i),
            Token::Op(o) => format!("operator `{}`", o),
            Token::Lit(l) => format!("literal `{}`", l),
        };

        write!(f, "{}", message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResIden {
    Macro,
    Type,
    If,
    Match,
    Mod,
}

impl ResIden {
    pub fn try_new(name: &str) -> Option<ResIden> {
        use ResIden::*;
        Some(match name {
            "macro" => Macro,
            "type" => Type,
            "if" => If,
            "match" => Match,
            "mod" => Mod,
            _ => {
                return None;
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResOp {
    Assign,
    Lambda,
    Equal,
    Pow,
    Compose,
    Field,
    Is,
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
            "|>" => Compose,
            "." => Field,
            ":" => Is,
            "," => Pair,
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,
            "%" => Rem,
            _ => {
                return None;
            },
        })
    }
}
