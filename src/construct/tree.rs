use crate::common::{
    data::Data,
    span::Spanned,
};
use crate::construct::{
    symbol::{SharedSymbol, UniqueSymbol},
    scope::Scope,
};

// symbol      - ast, cst, sst
// data        - ast, cst, sst
// block       - ast, cst, sst
// label       - ast, cst, sst
// tuple       - ast, cst, sst
// assign      - ast, cst, sst
// ffi         - ast, cst, sst
// lambda      - ast, cst
// form        - ast
// group       - ast
// pattern     - ast
// argpattern  - ast
// record      - ast
// is          - ast
// comp - ast
// syntax      - ast
// type        - ast
// call        -      cst, sst
// scoped_lmd  -           sst

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern<S> {
    Symbol(S),
    Data(Data),
    Label(Spanned<S>, Box<Spanned<Self>>),
    Tuple(Vec<Spanned<Self>>),
}

// TODO: impls for boxed items.

#[derive(Debug, Clone, PartialEq)]
pub enum Base<T, S> {
    Symbol(S),
    Data(Data),
    Label(Spanned<S>, Box<T>),
    Tuple(Vec<T>),

    Block(Vec<T>),
    Call(Box<T>, Box<T>), // fun, arg
    Assign(Spanned<Pattern<S>>, Box<T>),
    FFI(String, Box<T>),
}

impl<T, S> Base<T, S> {
    pub fn label(symbol: Spanned<S>, tree: T) -> Self {
        Base::Label(symbol, Box::new(tree))
    }

    pub fn call(fun: T, arg: T) -> Self {
        Base::Call(Box::new(fun), Box::new(arg))
    }

    pub fn assign(pat: Spanned<Pattern<S>>, expr: T) -> Self {
        Base::Assign(pat, Box::new(expr))
    }

    pub fn ffi(name: &str, expr: T) -> Self {
        Base::FFI(name.to_string(), Box::new(expr))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgPattern<S> {
    Keyword(S),
    Symbol(S),
    Group(Vec<Self>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Syntax<T, S> {
    arg_pat: Spanned<ArgPattern<S>>,
    body:    Box<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sugar<T, S> {
    Group(Box<T>),
    Form(Vec<T>),
    Pattern(Pattern<S>),
    ArgPattern(ArgPattern<S>),
    // Record,
    // Is,
    Comp(Box<T>, Box<T>), // arg, function
    Syntax(Syntax<T, S>),
}

impl<T, S> Sugar<T, S> {
    pub fn group(tree: T) -> Self {
        Sugar::Group(Box::new(tree))
    }

    pub fn comp(arg: T, fun: T) -> Self {
        Sugar::Comp(Box::new(arg), Box::new(fun))
    }

    pub fn syntax(arg_pat: Spanned<ArgPattern<S>>, tree: T) -> Self {
        Sugar::Syntax(Syntax { arg_pat, body: Box::new(tree) })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda<T> {
    arg:  Spanned<Pattern<SharedSymbol>>,
    body: Box<T>,
}

impl<T> Lambda<T> {
    pub fn new(arg: Spanned<Pattern<SharedSymbol>>, tree: T) -> Self {
        Lambda { arg, body: Box::new(tree) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    Base(Base<Spanned<AST>, SharedSymbol>),
    Sugar(Sugar<Spanned<AST>, SharedSymbol>),
    Lambda(Lambda<Spanned<AST>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CST {
    Base(Base<Spanned<CST>, SharedSymbol>),
    Lambda(Lambda<Spanned<CST>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopedLambda<T> {
    arg:   Spanned<Pattern<UniqueSymbol>>,
    body:  Box<T>,
    scope: Scope,
}

impl<T> ScopedLambda<T> {
    pub fn new(arg: Spanned<Pattern<UniqueSymbol>>, tree: T, scope: Scope) -> Self {
        ScopedLambda { arg, body: Box::new(tree), scope }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SST {
    Base(Base<Spanned<SST>, UniqueSymbol>),
    ScopedLambda(ScopedLambda<Spanned<SST>>)
}
