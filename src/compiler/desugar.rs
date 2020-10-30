use std::collections::HashMap;

use crate::common::span::{Span, Spanned};
use crate::compiler::{
    ast::{AST, Pattern},
    cst::CST,
    syntax::Syntax
};

// TODO: separate macro step and desugaring into two different steps?

pub fn desugar(ast: Spanned<AST>) -> Result<Spanned<CST>, Syntax> {
    let mut transformer = Transformer::new();
    let cst = transformer.walk(ast)?;
    return Ok(cst);
}

// TODO: add context for macro application
// NOTE: add spans?

#[derive(Debug, Clone)]
pub struct Rule {
    arg_pat: Vec<Spanned<Pattern>>,
    tree: Spanned<AST>,
}

impl Rule {
    /// Builds a new rule, making sure the rule's signature is valid
    pub fn new(
        arg_pat: Vec<Spanned<Pattern>>,
        tree: Spanned<AST>,
    ) -> Option<Rule> {
        let mut signature = vec![];
        for pat in arg_pat.iter() {
            if let Pattern::Keyword(name) = &pat.item {
                signature.push(name.clone())
            }
        }

        if signature.is_empty() { return None; }
        return Some(Rule { signature, arg_pat, tree })
    }

    pub fn match_form(
        &self,
        tree: Vec<Spanned<AST>>
    ) -> Option<Result<HashMap<String, Spanned<AST>>, Syntax>> {

    }
}

/// Applies compile-time transformations to the AST
pub struct Transformer {
    rules: Vec<Spanned<Rule>>,
}

impl Transformer {
    pub fn new() -> Transformer {
        Transformer { rules: vec![] }
    }

    // TODO: just pass the pattern and destructure during the gen pass?
    /// This function takes a pattern and converts it into an AST.
    pub fn depattern(pattern: Spanned<Pattern>) -> Result<Spanned<CST>, Syntax> {
        let cst = match pattern.item {
            Pattern::Symbol(s) => CST::Symbol(s),
            _ => Err(Syntax::error(
                "Pattern used that has not yet been implemented",
                pattern.span.clone(),
            ))?,
        };

        return Ok(Spanned::new(cst, pattern.span))
    }

    /// desugars an AST into a CST
    /// This function will become more complicated later on
    /// once macros are introduced, but right now it's basically a 1 to 1 translation
    pub fn walk(&mut self, ast: Spanned<AST>) -> Result<Spanned<CST>, Syntax> {
        let cst: CST = match ast.item {
            AST::Symbol(s) => CST::Symbol(s),
            AST::Data(d) => CST::Data(d),
            AST::Block(b) => self.block(b)?,
            AST::Form(f) => self.form(f)?,
            AST::Pattern(_) => unreachable!("Raw Pattern should not be in AST after parsing"),
            AST::Syntax { patterns, expression } => self.rule(*patterns, *expression)?,
            AST::Assign { pattern, expression } => self.assign(*pattern, *expression)?,
            AST::Lambda { pattern, expression } => self.lambda(*pattern, *expression)?,
            AST::Print(e) => CST::Print(Box::new(self.walk(*e)?)),
            AST::Label(n, e) => CST::Label(n, Box::new(self.walk(*e)?)),
        };

        return Ok(Spanned::new(cst, ast.span))
    }

    /// Recursively build up a call from a flat form.
    /// Basically turns `(a b c d)` into `(((a b) c) d)`.
    pub fn call(&mut self, mut f: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
        if f.len() < 1 {
            unreachable!("A call must have at least two values - a function and an expression")
        } else if f.len() == 1 {
            return Ok(self.walk(f[0].clone())?.item);
        }

        if f.len() == 2 {
            let arg = f.pop().unwrap();
            let fun = f.pop().unwrap();
            Ok(CST::call(self.walk(fun)?, self.walk(arg)?))
        } else {
            let arg = self.walk(f.pop().unwrap())?;
            let f_span = Span::join(f.iter().map(|e| e.span.clone()).collect::<Vec<Span>>());
            Ok(CST::call(Spanned::new(self.call(f)?, f_span), arg))
        }
    }

    // TODO: do raw apply and check once macros get more complicated.
    // TODO: Make it possible for forms with less than one value to exist
    pub fn form(&mut self, mut form: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
        // collect all in-scope pseudokeywords


        // convert symbols to in-scope pseudokeywords
        form = form.iter()
            .map(|branch| match branch.item {
                AST::Symbol(name)
            })
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

    pub fn rule(&mut self, arg_pat: Spanned<Vec<Spanned<Pattern>>>, tree: Spanned<AST>) -> Result<CST, Syntax> {
        let patterns_span = arg_pat.span.clone();

        let rule = Rule::new(arg_pat.item, tree)
            .ok_or(Syntax::error(
                "Syntactic macros must have at least one pseudokeyword",
                patterns_span.clone(),
            ))?;

        for defined in self.rules.iter() {
            if defined.item.signature == rule.signature {
                return Err(Syntax::error(
                    &format!(
                        "Syntactic macro with the signature {} has already been defined:\n{}",
                        rule.display_signature(),
                        defined.span,
                    ),
                    patterns_span,
                ));
            }
        }

        self.rules.push(Spanned::new(rule, patterns_span));

        // TODO: return nothing?
        Ok(CST::Block(vec![]))
    }
}
