use crate::common::span::{Span, Spanned};
use crate::compiler::{
    ast::{AST, Pattern},
    cst::CST,
    syntax::Syntax
};

pub fn desugar(ast: Spanned<AST>) -> Result<Spanned<CST>, Syntax> {
    let mut transformer = Transformer::new();
    let cst = transformer.walk(ast)?;
    return Ok(cst);
}

// TODO: add context for macro application

pub struct Rule {
    arg_pat: Vec<Spanned<Pattern>>,
    tree: Spanned<AST>,
}

/// Applies compile-time transformations to the AST
pub struct Transformer {
    rules: Vec<Rule>,
}

impl Transformer {
    pub fn new() -> Transformer {
        Transformer {
            rules: vec![]
        }
    }

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
    pub fn walk(&mut self, ast: Spanned<AST>) -> Result<Spanned<CST>, Syntax> {
        let cst: CST = match ast.item {
            AST::Symbol => CST::Symbol(ast.span.contents()),
            AST::Data(d) => CST::Data(d),
            AST::Block(b) => self.block(b)?,
            AST::Form(f) => self.form(f)?,
            AST::Pattern(_) => unreachable!("Raw Pattern should not be in AST after desugaring"),
            AST::Syntax { patterns, expression } => self.rule(*patterns, *expression)?,
            AST::Assign { pattern, expression } => self.assign(*pattern, *expression)?,
            AST::Lambda { pattern, expression } => self.lambda(*pattern, *expression)?,
            AST::Print(e) => CST::Print(Box::new(self.walk(*e)?)),
            AST::Label(n, e) => CST::Label(n, Box::new(self.walk(*e)?)),
        };

        return Ok(Spanned::new(cst, ast.span))
    }

    pub fn form(&mut self, mut f: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
        if f.len() < 2 {
            unreachable!("A call must have at least two values - a function and an expression")
        }

        if f.len() == 2 {
            let arg = f.pop().unwrap();
            let fun = f.pop().unwrap();
            Ok(CST::call(self.walk(fun)?, self.walk(arg)?))
        } else {
            let arg = self.walk(f.pop().unwrap())?;
            let f_span = Span::join(f.iter().map(|e| e.span.clone()).collect::<Vec<Span>>());
            Ok(CST::call(self.walk(Spanned::new(AST::Form(f), f_span))?, arg))
        }
    }

    pub fn block(&mut self, b: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
        let mut expressions = vec![];
        for e in b {
            expressions.push(self.walk(e)?)
        }

        Ok(CST::Block(expressions))
    }

    pub fn assign(&mut self, p: Spanned<Pattern>, e: Spanned<AST>) -> Result<CST, Syntax> {
        todo!()
    }

    pub fn lambda(&mut self, p: Spanned<Pattern>, e: Spanned<AST>) -> Result<CST, Syntax> {
        todo!()
    }

    pub fn rule(&mut self, arg_pat: Vec<Spanned<Pattern>>, tree: Spanned<AST>) -> Result<CST, Syntax> {
        return Err(Syntax::error(
            "Syntactic macros are not yet supported",
            tree.span.clone(),
        ));

        let rule = Rule { arg_pat, tree };
        self.rules.push(rule);
        // TODO: return nothing?
        // TODO: check that pattern is correct.
        // i.e. contains at least one keyword,
        // and keywords are not already in use.
        Ok(CST::Block(vec![]))
    }
}
