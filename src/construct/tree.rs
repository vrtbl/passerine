use std::convert::TryFrom;

use crate::common::{lit::Lit, span::Spanned};
use crate::construct::{
    scope::Scope,
    symbol::{SharedSymbol, UniqueSymbol},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern<S> {
    Symbol(S),
    Lit(Lit),
    Label(Spanned<S>, Box<Spanned<Self>>),
    Tuple(Vec<Spanned<Self>>),
    Chain(Vec<Spanned<Self>>),
}

// TODO: impls for boxed items.

#[derive(Debug, Clone, PartialEq)]
pub enum Base<T, S> {
    Symbol(S),
    Label(S),
    Lit(Lit),
    Tuple(Vec<T>),
    Module(Box<T>),

    Block(Vec<T>),
    Call(Box<T>, Box<T>), // fun, arg
    Assign(Spanned<Pattern<S>>, Box<T>),
    FFI(usize, Box<T>),
}

impl<T, S> Base<T, S> {
    pub fn call(fun: T, arg: T) -> Self {
        Base::Call(Box::new(fun), Box::new(arg))
    }

    pub fn assign(pat: Spanned<Pattern<S>>, expr: T) -> Self {
        Base::Assign(pat, Box::new(expr))
    }

    // pub fn ffi(name: &str, expr: T) -> Self {
    //     Base::FFI(name.to_string(), Box::new(expr))
    // }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sugar<T, S> {
    Group(Box<T>),
    Form(Vec<T>),
    Pattern(Pattern<S>),
    // Record,
    Is(Box<T>, Box<T>),   // expr, type
    Comp(Box<T>, Box<T>), // arg, function
    Field(Box<T>, Box<T>), // struct, field
                          // TODO: math operators
}

impl<T, S> Sugar<T, S> {
    pub fn group(tree: T) -> Self {
        Sugar::Group(Box::new(tree))
    }

    pub fn is(expr: T, ty: T) -> Self {
        Sugar::Is(Box::new(expr), Box::new(ty))
    }

    pub fn comp(arg: T, fun: T) -> Self {
        Sugar::Comp(Box::new(arg), Box::new(fun))
    }

    pub fn field(record: T, name: T) -> Self {
        Sugar::Field(Box::new(record), Box::new(name))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda<T> {
    arg: Spanned<Pattern<SharedSymbol>>,
    body: Box<T>,
}

impl<T> Lambda<T> {
    pub fn new(arg: Spanned<Pattern<SharedSymbol>>, tree: T) -> Self {
        Lambda {
            arg,
            body: Box::new(tree),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    Base(Base<Spanned<AST>, SharedSymbol>),
    Sugar(Sugar<Spanned<AST>, SharedSymbol>),
    Lambda(Lambda<Spanned<AST>>),
}

impl TryFrom<AST> for Pattern<SharedSymbol> {
    type Error = String;

    /// Tries to convert an `AST` into a `Pattern`.
    /// Patterns mirror the `AST`s they are designed to destructure.
    /// During parsing, they are just parsed as `AST`s -
    /// When the compiler can determine that an AST is actually a pattern,
    /// It performs this conversion.
    fn try_from(ast: AST) -> Result<Self, Self::Error> {
        // if true { todo!("SharedSymbol lookup"); }
        Ok(match ast {
            AST::Base(Base::Symbol(s)) => Pattern::Symbol(s),
            AST::Base(Base::Lit(d)) => Pattern::Lit(d),
            AST::Base(Base::Label(k)) => Err(format!(
                "This Label used in a pattern does not unwrap any data.\n\
                    To match a Label and ignore its contents, use `{:?} _`",
                k,
            ))?,
            AST::Base(Base::Tuple(t)) => {
                let mut patterns = vec![];
                for item in t {
                    patterns.push(item.map(Pattern::try_from)?);
                }
                Pattern::Tuple(patterns)
            },

            AST::Sugar(Sugar::Pattern(p)) => p,
            AST::Sugar(Sugar::Form(f)) => {
                let mut patterns = vec![];
                for item in f {
                    patterns.push(item.map(Pattern::try_from)?);
                }
                Pattern::Chain(patterns)
            },
            AST::Sugar(Sugar::Group(e)) => e.map(Pattern::try_from)?.item,
            _ => Err("Unexpected construct inside pattern")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CST {
    Base(Base<Spanned<CST>, SharedSymbol>),
    Lambda(Lambda<Spanned<CST>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopedLambda<T> {
    arg: Spanned<Pattern<UniqueSymbol>>,
    body: Box<T>,
    scope: Scope,
}

impl<T> ScopedLambda<T> {
    pub fn new(
        arg: Spanned<Pattern<UniqueSymbol>>,
        tree: T,
        scope: Scope,
    ) -> Self {
        ScopedLambda {
            arg,
            body: Box::new(tree),
            scope,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SST {
    Base(Base<Spanned<SST>, UniqueSymbol>),
    ScopedLambda(ScopedLambda<Spanned<SST>>),
}
