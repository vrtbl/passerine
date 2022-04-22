use std::fmt::Display;

use crate::common::{
    lit::Lit,
    span::Spanned,
};

#[derive(Debug, Clone, PartialEq, proptest_derive::Arbitrary)]
pub enum Delim {
    Paren,
    Curly,
    Square,
}

impl Display for Delim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Delim::Paren => "parenthesis",
            Delim::Curly => "curly brackets",
            Delim::Square => "square brackets",
        };

        write!(f, "{}", name)
    }
}

pub type Tokens = Vec<Spanned<Token>>;

#[derive(Debug, Clone, PartialEq, proptest_derive::Arbitrary)]
pub enum Token {
    // Grouping
    Open(Delim),
    Close(Delim),
    Sep,

    // Leafs
    Iden(String),
    Label(String),
    Op(String),
    Lit(Lit),
}

pub type TokenTrees = Vec<Spanned<TokenTree>>;

/// These are the different tokens the lexer will output.
/// `Token`s with data contain that data,
/// e.g. a boolean will be a `Lit::Boolean(...)`, not just a string.
/// `Token`s can be spanned using `Spanned<Token>`.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenTree {
    // Grouping
    Block(Vec<Spanned<TokenTrees>>),
    List(TokenTrees),
    Form(TokenTrees),

    // Leafs
    Iden(String),
    Label(String),
    Op(String),
    Lit(Lit),
}

impl Display for TokenTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // pretty formatting for tokens
        // just use debug if you're not printing a message or something.
        use TokenTree::*;
        let message = match self {
            Block(_) => "tokens grouped by curly brackets".to_string(),
            List(_) => "tokens grouped by square brackets".to_string(),
            Form(_) => "a group of tokens".to_string(),
            Iden(i) => format!("identifier `{}`", i),
            Label(i) => format!("type identifier `{}`", i),
            Op(o) => format!("operator `{}`", o),
            Lit(l) => format!("literal `{}`", l),
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
