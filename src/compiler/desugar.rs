use crate::common::span::{Span, Spanned};
use crate::compiler::{
    ast::AST,
    cst::CST,
    syntax::Syntax
};

// TODO: move branches into separate functions
// TODO: add context for macro application

/// desugars an AST into a DST
/// This function will become more complicated later on
/// once macros are introduced, but right now it's basically a 1 to 1 translation
pub fn desugar(ast: Spanned<AST>) -> Result<Spanned<CST>, Syntax> {
    let cst: CST = match ast.item {
        AST::Symbol => CST::Symbol,
        AST::Data(d) => CST::Data(d),
        AST::Block(b) => CST::Block({
            let mut expressions = vec![];
            for e in b { expressions.push(desugar(e)?) }
            expressions
        }),
        AST::Pattern(_p) => unimplemented!("patterns are not yet implemented"),
        AST::Form(mut f) => {
            if f.len() < 2 {
                // maybe just error?
                unreachable!("A call must have at least two values - a function and an expression")
            }

            if f.len() == 2 {
                let arg = f.pop().unwrap();
                let fun = f.pop().unwrap();
                CST::call(desugar(fun)?, desugar(arg)?)
            } else {
                let arg = desugar(f.pop().unwrap())?;
                let f_span = Span::join(f.iter().map(|e| e.span.clone()).collect::<Vec<Span>>());
                CST::call(desugar(Spanned::new(AST::Form(f), f_span))?, arg)
            }
        }
        AST::Assign { pattern, expression } => CST::assign(desugar(*pattern)?, desugar(*expression)?),
        AST::Lambda { pattern, expression } => CST::lambda(desugar(*pattern)?, desugar(*expression)?),
        AST::Print(e) => CST::Print(Box::new(desugar(*e)?)),
        AST::Label(n, e) => CST::Label(n, Box::new(desugar(*e)?)),
    };

    return Ok(Spanned::new(cst, ast.span))
}
