use crate::common::span::{Span, Spanned};
use crate::compiler::{
    ast::{AST, Pattern},
    cst::CST,
    syntax::Syntax
};

// TODO: move branches into separate functions
// TODO: add context for macro application
// TODO: move to impl Transformer

pub fn depattern(pattern: Spanned<Pattern>) -> Result<Spanned<CST>, Syntax> {
    let cst = match pattern.item {
        Pattern::Symbol => CST::Symbol(pattern.span.contents()),
        _ => Err(Syntax::error("Pattern used that has not yet been implemented", pattern.span.clone()))?,
    };

    return Ok(Spanned::new(cst, pattern.span))
}

/// desugars an AST into a CST
/// This function will become more complicated later on
/// once macros are introduced, but right now it's basically a 1 to 1 translation
pub fn desugar(ast: Spanned<AST>) -> Result<Spanned<CST>, Syntax> {
    let cst: CST = match ast.item {
        AST::Symbol => CST::Symbol(ast.span.contents()),
        AST::Data(d) => CST::Data(d),
        AST::Block(b) => block(b)?,
        AST::Form(f) => form(f)?,
        AST::Pattern(_) => unreachable!("Raw Pattern should not be in AST after desugaring"),
        AST::Syntax { .. } => unreachable!("Unexpanded Syntax rules should not be in AST after desugaring"),
        AST::Assign { pattern, expression } => assign(*pattern, *expression)?,
        AST::Lambda { pattern, expression } => lambda(*pattern, *expression)?,
        AST::Print(e) => CST::Print(Box::new(desugar(*e)?)),
        AST::Label(n, e) => CST::Label(n, Box::new(desugar(*e)?)),
    };

    return Ok(Spanned::new(cst, ast.span))
}

pub struct Rule {
    arg_pat: Vec<Spanned<Pattern>>,
    tree: Spanned<AST>,
}

/// Applies compile-time transformations to the AST
pub struct Transformer {
    rules: Vec<Rule>,
}

pub fn form(mut f: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
    if f.len() < 2 {
        unreachable!("A call must have at least two values - a function and an expression")
    }

    if f.len() == 2 {
        let arg = f.pop().unwrap();
        let fun = f.pop().unwrap();
        Ok(CST::call(desugar(fun)?, desugar(arg)?))
    } else {
        let arg = desugar(f.pop().unwrap())?;
        let f_span = Span::join(f.iter().map(|e| e.span.clone()).collect::<Vec<Span>>());
        Ok(CST::call(desugar(Spanned::new(AST::Form(f), f_span))?, arg))
    }
}

pub fn block(b: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
    let mut expressions = vec![];
    for e in b {
        expressions.push(desugar(e)?)
    }

    Ok(CST::Block(expressions))
}

pub fn assign(p: Spanned<Pattern>, e: Spanned<AST>) -> Result<CST, Syntax> {
    todo!()
}

pub fn lambda(p: Spanned<Pattern>, e: Spanned<AST>) -> Result<CST, Syntax> {
    todo!()
}
