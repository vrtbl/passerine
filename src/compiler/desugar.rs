use std::collections::HashMap;

use crate::common::span::{Span, Spanned};
use crate::compiler::{
    ast::{AST, Pattern, ArgPat},
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
pub enum Bind {
    Nothing,
    // name, bound sub-tree
    Pair(String, Spanned<AST>),
    // bindings, unmatched sub-trees
    Group(HashMap<String, Spanned<AST>>, Vec<Spanned<AST>>),
}

// type BindRemaining = (Vec<(String, Spanned<AST>)>, Option<Spanned<AST>>);

#[derive(Debug, Clone)]
pub struct Rule {
    arg_pat: Spanned<ArgPat>,
    tree: Spanned<AST>,
}

impl Rule {
    /// Builds a new rule, making sure the rule's signature is valid
    pub fn new(
        arg_pat: Spanned<ArgPat>,
        tree: Spanned<AST>,
    ) -> Rule {
        return Rule { arg_pat, tree }
    }

    /// Binds a form subgroup without recursively matching the form itself.
    pub fn bind_group(
        pats: Vec<Spanned<ArgPat>>,
        mut remaining: Vec<Spanned<AST>>,
    ) -> Result<Bind, Syntax> {
        println!("binding group");
        let mut bindings = HashMap::new();

        for pat in pats {
            let span = pat.span.clone();
            let next = remaining.pop().unwrap();

            let bind = if let AST::Form(_) = next.item {
                Rule::bind(pat, Spanned::new(
                    AST::Form(remaining.to_vec()),
                    Spanned::build(&remaining))
                )?
            } else {
                Rule::bind(pat, next)?
            };

            let collision = match bind {
                Bind::Nothing => None,
                Bind::Pair(k, v) => bindings.insert(k, v),
                Bind::Group(new_bindings, r) => {
                    // TODO: maybe use .extend? we need to err on collisions,
                    // but .extend seems to silently ignore these.
                    remaining = r;
                    let mut collision = None;
                    for (k, v) in new_bindings {
                        if let c @ Some(_) = bindings.insert(k, v) {
                            collision = c;
                            break;
                        }
                    }
                    collision
                },
            };

            if let Some(ast) = collision {
                return Err(Syntax::error(
                    &format!(
                        "While expanding macro\n\
                        {}\n\
                        Variable has already been declared in syntactic macro argument pattern",
                        span
                    ),
                    ast.span,
                ));
            }
        }

        Ok(Bind::Group(bindings, remaining))
    }

    /// Binds an AST to a macro.
    pub fn bind(
        arg_pat: Spanned<ArgPat>,
        tree: Spanned<AST>,
    ) -> Result<Bind, Syntax> {
        println!("binding");
        match arg_pat.item {
            ArgPat::Symbol(name) => Ok(Bind::Pair(name, tree)),

            ArgPat::Keyword(expected) => {
                match tree.item {
                    AST::Symbol(name) if name == expected => Ok(Bind::Nothing),
                    _ => return Err(Syntax::error(
                        &format!("Expected the pseudokeyword '{} while binding macro", expected),
                        tree.span.clone(),
                    )),
                }
            },

            ArgPat::Group(pats) => {
                let remaining: Vec<Spanned<AST>> = match tree.item {
                    AST::Form(f) => f.into_iter().rev().collect(),
                    _ => {
                        println!("{:#?}", tree.item);
                        unreachable!("Expected a form");
                    },
                };
                Rule::bind_group(pats, remaining)
            },
        }
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
            AST::ArgPat(_) => unreachable!("Raw Argument Pattern should not be in AST after parsing"),
            AST::Syntax { arg_pat, expression } => self.rule(*arg_pat, *expression)?,
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
        // form = form.iter()
        //     .map(|branch| match branch.item {
        //         AST::Symbol(name)
        //     })

        for rule in self.rules.iter() {
            let binding = Rule::bind(rule.item.arg_pat.clone(), Spanned::new(AST::Form(form.clone()), Spanned::build(&form)))?;
            println!("{:#?}", binding);
        }

        todo!()
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

    pub fn rule(&mut self, arg_pat: Spanned<ArgPat>, tree: Spanned<AST>) -> Result<CST, Syntax> {
        let patterns_span = arg_pat.span.clone();

        // TODO: check that rule is valid
        let rule = Rule::new(arg_pat, tree);

        // TODO: check for conflicting macros
        self.rules.push(Spanned::new(rule, patterns_span));

        // TODO: return nothing?
        Ok(CST::Block(vec![]))
    }
}
