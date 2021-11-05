use std::{
    fmt::Display,
    rc::Rc,
};

use crate::common::{
    span::Spanned,
    lit::Lit,
};

// TODO: remove associated data from tokens.

pub type Tokens = Vec<Spanned<Token>>;

/// These are the different tokens the lexer will output.
/// `Token`s with data contain that data,
/// e.g. a boolean will be a `Lit::Boolean(...)`, not just a string.
/// `Token`s can be spanned using `Spanned<Token>`.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Delimiters
    // TODO: do not reference count!
    Delim(Delim, Rc<Tokens>),

    // Names
    Label(String),
    Iden(String),
    Op(String),

    // Values
    Lit(Lit),

    // Context
    Sep,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // pretty formatting for tokens
        // just use debug if you're not printing a message or something.
        let message = match self {
            Token::Delim(delim, _) => format!("tokens grouped by {}", delim),
            Token::Label(l)        => format!("the label `{}`", l),
            Token::Iden(i)         => format!("the identifier `{}`", i),
            Token::Op(o)           => format!("the operator `{}`", o),
            Token::Lit(l)          => format!("the literal `{}`", l),
            Token::Sep             => "a separator".to_string(),
        };

        write!(f, "{}", message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delim {
    // List of forms
    Curly,
    Paren,
    Square,
}

impl Display for Delim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Delim::Paren  => "parenthesis",
            Delim::Curly  => "curly bracket",
            Delim::Square => "square bracket",
        };

        write!(f, "{}", message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            "type"  => Type,
            "if"    => If,
            "match" => Match,
            "mod"   => Mod,
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
