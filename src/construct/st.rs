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
// composition - ast
// syntax      - ast
// type        - ast
// call        -      cst, sst
// scoped_lmd  -           sst

pub enum Pattern<S> {
    Symbol(S),
    Data(Data),
    Label(S, Box<Spanned<Self>>),
    Tuple(Vec<Spanned<Self>>),
}

pub enum Base<T, S> {
    Symbol(S),
    Data(Data),
    Label(S, Box<T>),
    Tuple(Vec<T>),

    Block(Vec<T>),
    Call(Box<T>, Box<T>), // fun, arg
    Assign(Pattern<S>, Box<T>),
}

pub enum ArgPattern<S> {
    Keyword(S),
    Symbol(S),
    Group(Vec<Self>),
}

pub struct Syntax<T, S> {
    argpat: ArgPattern<S>,
    body:   Box<T>,
}

pub enum Sugar<T, S> {
    Group(Box<T>),
    Pattern(Pattern<S>),
    ArgPattern(ArgPattern<S>),
    // Record,
    // Is,
    Composition(Box<T>, Box<T>), // arg, function
    Syntax(Syntax<T, S>),
}

pub struct Lambda<T, S> {
    arg: Pattern<S>,
    body: Box<T>,
}

pub enum AST {
    Base(Base<AST, SharedSymbol>),
    Sugar(Sugar<AST, SharedSymbol>),
    Lambda(Lambda<AST, SharedSymbol>),
}

pub enum CST {
    Base(Base<CST, SharedSymbol>),
    Lambda(Lambda<CST, SharedSymbol>),
}

pub struct ScopedLambda<T> {
    arg: Pattern<UniqueSymbol>,
    body: Box<T>,
    scope: Scope,
}

pub enum SST {
    Base(Base<SST, UniqueSymbol>),
    ScopedLambda(ScopedLambda<SST>)
}
