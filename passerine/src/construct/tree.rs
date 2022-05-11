use std::convert::TryFrom;

use crate::{
    common::{
        lit::Lit,
        span::Spanned,
    },
    construct::{
        scope::Scope,
        symbol::{
            SharedSymbol,
            UniqueSymbol,
        },
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern<S> {
    Symbol(S),
    Lit(Lit),
    Label(Spanned<S>, Box<Spanned<Self>>),
    Tuple(Vec<Spanned<Self>>),
    Chain(Vec<Spanned<Self>>),
}

impl<S> Pattern<S> {
    pub fn label(symbol: Spanned<S>, pattern: Spanned<Self>) -> Self {
        Pattern::Label(symbol, Box::new(pattern))
    }

    pub fn map<Z>(self, symbol: impl Fn(S) -> Z) -> Pattern<Z> {
        match self {
            Pattern::Symbol(s) => Pattern::Symbol(symbol(s)),
            Pattern::Lit(l) => Pattern::Lit(l),
            Pattern::Label(s, p) => {
                todo!();
                // Pattern::label(s, p)
            },
            Pattern::Tuple(t) => todo!(),
            Pattern::Chain(c) => Pattern::Chain(
                todo!(),
                // c.into_iter()
                //     .map(|s| s.map(move |s| s.map(symbol)))
                //     .collect(), /* todo */
            ),
        }
    }
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

    pub fn module(module: T) -> Self { Base::Module(Box::new(module)) }

    // pub fn ffi(name: &str, expr: T) -> Self {
    //     Base::FFI(name.to_string(), Box::new(expr))
    // }

    pub fn map<Y, Z>(
        self,
        tree: impl Fn(T) -> Y,
        symbol: impl Fn(S) -> Z,
    ) -> Base<Y, Z> {
        match self {
            Base::Symbol(s) => Base::Symbol(symbol(s)),
            Base::Label(l) => Base::Label(symbol(l)),
            Base::Lit(l) => Base::Lit(l),
            Base::Tuple(t) => Base::Tuple(t.into_iter().map(tree).collect()),
            Base::Module(m) => Base::module(tree(*m)),
            Base::Block(b) => Base::Block(b.into_iter().map(tree).collect()),
            Base::Call(f, a) => Base::call(tree(*f), tree(*a)),
            Base::Assign(s, e) => {
                // todo!()
                let fun = |p: Pattern<S>| p.map(symbol);
                Base::assign(s.map(fun), tree(*e))
            },
            Base::FFI(_, _) => todo!("FFI is depracated! !!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sugar<T> {
    Group(Box<T>),
    Form(Vec<T>),
    // Pattern(Pattern<S>),
    // Record,
    Is(Box<T>, Box<T>), // expr, type
    // A function composition
    Comp(Box<T>, Box<T>), // arg, function
    Field(Box<T>, Box<T>), /* struct, field
                           * TODO: math operators */
}

impl<T> Sugar<T> {
    pub fn group(tree: T) -> Self { Sugar::Group(Box::new(tree)) }

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
    pub arg:  Spanned<Pattern<SharedSymbol>>,
    pub body: Box<T>,
}

impl<T> Lambda<T> {
    pub fn new(arg: Spanned<Pattern<SharedSymbol>>, body: T) -> Self {
        Lambda {
            arg,
            body: Box::new(body),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    Base(Base<Spanned<AST>, SharedSymbol>),
    Sugar(Sugar<Spanned<AST>>),
    Lambda(Lambda<Spanned<AST>>),
}

impl TryFrom<AST> for Pattern<SharedSymbol> {
    type Error = String;

    /// Tries to convert an `AST` into a `Pattern`.
    /// Patterns mirror the `AST`s they are designed to
    /// destructure. During parsing, they are just
    /// parsed as `AST`s - When the compiler can
    /// determine that an AST is actually a pattern,
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
                    patterns.push(item.try_map(Pattern::try_from)?);
                }
                Pattern::Tuple(patterns)
            },

            // AST::Sugar(Sugar::Pattern(p)) => p,
            AST::Sugar(Sugar::Form(f)) => {
                let mut patterns = vec![];
                for item in f {
                    patterns.push(item.try_map(Pattern::try_from)?);
                }
                Pattern::Chain(patterns)
            },
            AST::Sugar(Sugar::Group(e)) => e.try_map(Pattern::try_from)?.item,
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
    pub arg:   Spanned<Pattern<UniqueSymbol>>,
    pub body:  Box<T>,
    pub scope: Scope,
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
