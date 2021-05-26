use std::{
    convert::TryFrom,
    collections::{HashMap, HashSet},
};

use crate::common::span::{Span, Spanned};

use crate::compiler::{lower::Lower, syntax::Syntax};

use crate::construct::{
    rule::Rule,
    ast::{AST, ASTPattern, ArgPattern},
    cst::{CST, CSTPattern},
    symbol::SharedSymbol,
    module::{ThinModule, Module},
};

impl Lower for Module<Spanned<AST>, usize> {
    type Out = ThinModule<Spanned<CST>>;

    /// Desugares an `AST` into a `CST`, applying macro transformations along the way.
    fn lower(self) -> Result<Self::Out, Syntax> {
        println!("{:#?}", self.repr);
        let mut transformer = Transformer::new(self.assoc);
        let cst = transformer.walk(self.repr)?;
        return Ok(ThinModule::thin(cst));
    }
}

// TODO: separate macro step and desugaring into two different steps?

// pub fn desugar(ast: Module<Spanned<AST>, usize>) -> Result<Module<Spanned<CST>, ()>, Syntax> {
//     let mut transformer = Transformer::new(ast.associated);
//     let cst = transformer.walk(ast.syntax_tree)?;
//     return Ok(cst);
// }

/// Applies compile-time transformations to the AST.
pub struct Transformer {
    // TODO: make this scoped to the current function
    rules:         Vec<Spanned<Rule>>,
    lowest_shared: usize,
    mangles:       HashMap<SharedSymbol, SharedSymbol>,
}

impl Transformer {
    /// Creates a new transformer with no macro transformation rules.
    pub fn new(lowest_shared: usize) -> Transformer {
        Transformer { rules: vec![], lowest_shared, mangles: HashMap::new() }
    }

    /// Desugars an `AST` into a `CST`,
    /// By walking over it in a fairly straight-forward manner.
    pub fn walk(&mut self, ast: Spanned<AST>) -> Result<Spanned<CST>, Syntax> {
        let cst: CST = match ast.item {
            AST::Symbol(_) => self.symbol(ast.clone())?,
            AST::Data(d) => CST::Data(d),
            AST::Block(b) => self.block(b)?,
            AST::Form(f) => self.form(f)?,
            AST::Group(a) => self.walk(*a)?.item,
            AST::Tuple(t) => self.tuple(t)?,
            AST::Pattern(_) => return Err(Syntax::error("Unexpected pattern", &ast.span)),
            AST::ArgPattern(_)  => return Err(Syntax::error("Unexpected argument pattern", &ast.span)),
            AST::Label(n, e) => CST::Label(n, Box::new(self.walk(*e)?)),
            AST::Syntax { arg_pat, expression } => self.rule(*arg_pat, *expression)?,
            AST::Assign { pattern, expression } => self.assign(*pattern, *expression)?,
            AST::Lambda { pattern, expression } => self.lambda(*pattern, *expression)?,
            AST::Composition { argument, function } => self.composition(*argument, *function)?,
            AST::FFI { name, expression } => self.ffi(name, *expression)?,
            AST::Record(_) => todo!(),
            AST::Is { .. } => todo!(),
        };

        return Ok(Spanned::new(cst, ast.span))
    }

    /// Converts a symbol.
    /// Note that symbols can be one-item macros;
    /// Function calls are always parsed with at least two items,
    /// So we need to wrap this symbol in a vec and interpret it as a form.
    pub fn symbol(&mut self, ast: Spanned<AST>) -> Result<CST, Syntax> {
        self.form(vec![ast])
    }

    /// Recursively build up a call from a flat form.
    /// Basically turns `(a b c d)` into `(((a b) c) d)`.
    pub fn call(&mut self, mut f: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
        match f.len() {
            0 => unreachable!("A call must have at least two values - a function and an expression"),
            1 => match f.pop().unwrap().item {
                AST::Symbol(name) => Ok(CST::Symbol(name)),
                _ => unreachable!("A non-symbol call of length 1 is can not be constructed")
            },
            2 => {
                let arg = f.pop().unwrap();
                let fun = f.pop().unwrap();
                Ok(CST::call(self.walk(fun)?, self.walk(arg)?))
            },
            _higher => {
                let arg = self.walk(f.pop().unwrap())?;
                // can't join, because some spans may be in macro
                let f_span = f[0].span.clone();
                Ok(CST::call(Spanned::new(self.call(f)?, f_span), arg))
            },
        }
    }

    // TODO: Make it possible for forms with zero values to exist?
    /// Desugars a form.
    /// This is where most of the macro logic resides.
    /// Applying a macro really happens in four broad strokes:
    /// 1. We match the form against all macros currently in scope.
    /// 2. If there was one match, we're done! we apply the macro and keep on going.
    /// 3. If there were no matches, we ensure that it couldn't've been a macro,
    ///    then parse it as a function call.
    /// 4. If there was more than one match, we point out the ambiguity.
    pub fn form(&mut self, form: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
        println!("{:#?}", form);

        // apply all the rules in scope
        let mut matches = vec![];
        for rule in self.rules.iter() {
            let mut reversed_remaining = form.clone().into_iter().rev().collect();
            let possibility = Rule::bind(
                &rule.item.arg_pat,
                &mut reversed_remaining,
                &mut self.lowest_shared,
                &mut self.mangles,
            );

            println!("{:#?}", possibility);

            if let Some(bindings) = possibility {
                println!("Match!");
                if reversed_remaining.is_empty() {
                    matches.push((rule, bindings?))
                } else {
                    println!("OK! {:?}", reversed_remaining);
                }
            }
        }

        println!("Matches: {:#?}", matches);

        // no macros were matched
        if matches.len() == 0 {
            // collect all in-scope pseudokeywords
            let mut pseudokeywords: HashMap<SharedSymbol, String> = HashMap::new();
            for rule in self.rules.iter() {
                for (pseudokeyword, name) in Rule::keywords(&rule.item.arg_pat) {
                    pseudokeywords.insert(pseudokeyword, name);
                }
            }

            // into a set for quick membership checking
            let potential_keywords = form.iter()
                .filter(|i| if let AST::Symbol(_) = i.item { true } else { false          })
                .map(   |i| if let AST::Symbol(s) = i.item { s    } else { unreachable!() })
                .collect::<HashSet<SharedSymbol>>();

            // calculate pseudokeyword collisions in case of ambiguity
            let intersection = potential_keywords
                .intersection(&pseudokeywords.keys().cloned().collect())
                .map(|s| *s).collect::<Vec<SharedSymbol>>();

            // no collisions? process the form as a function call instead
            if intersection.is_empty() { return self.call(form); }

            // a collision? point it out!
            return Err(Syntax::error(
                &format!(
                    "In-scope pseudokeyword{} {} used, but no macros match the form.",
                    if intersection.len() == 1 { "" } else { "s" },
                    intersection.iter()
                        .map(|kw| format!("'{}'", pseudokeywords.get(kw).unwrap()))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                &Spanned::build(&form),
            ))
        }

        // multiple macros were matched,
        // so this form is ambiguious
        if matches.len() > 1 {
            // TODO: make the error prettier
            // might have to rework Syntax a bit...
            return Err(Syntax::error(
                &format!(
                    "This form matched multiple macros:\n\n{}\
                    Note: A form may only match one macro, this must be unambiguious; \
                    try using variable names different than those of pseudokeywords currently in scope, \
                    Adjusting the definitions of locally-defined macros, \
                    or using parenthesis '( ... )' or curlies '{{ ... }}' to group nested macros.Sized",
                    matches.iter()
                        .map(|s| format!("{}", s.0.span))
                        .collect::<Vec<String>>()
                        .join(""),
                ),
                &Spanned::build(&form),
            ))
        }

        // apply the rule to apply the macro!
        let (rule, mut bindings) = matches.pop().unwrap();
        let expanded = Rule::expand(
            rule.item.tree.clone(),
            &mut bindings,
            &mut self.lowest_shared,
            &mut self.mangles
        )?;

        return Ok(self.walk(expanded)?.item);
    }

    /// Desugar a tuple.
    /// Nothing fancy here.
    pub fn tuple(&mut self, tuple: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
        let mut expressions = vec![];
        for expression in tuple {
            expressions.push(self.walk(expression)?)
        }

        Ok(CST::Tuple(expressions))
    }

    /// Desugar a function application.
    /// A composition takes the form `c . b . a`
    /// and is left-associative `(c . b) . a`.
    /// When desugared, the above is equivalent to the call `a b c`.
    pub fn composition(&mut self, argument: Spanned<AST>, function: Spanned<AST>) -> Result<CST, Syntax> {
        Ok(CST::call(self.walk(function)?, self.walk(argument)?))
    }

    /// Desugar a FFI call.
    /// We walk the expression that may be passed to the FFI.
    pub fn ffi(&mut self, name: String, expression: Spanned<AST>) -> Result<CST, Syntax> {
        Ok(CST::FFI { name, expression: Box::new(self.walk(expression)?) })
    }

    /// Desugars a block,
    /// i.e. a series of expressions that takes on the value of the last one.
    pub fn block(&mut self, block: Vec<Spanned<AST>>) -> Result<CST, Syntax> {
        let mut expressions = vec![];
        for expression in block {
            expressions.push(self.walk(expression)?)
        }

        Ok(CST::Block(expressions))
    }

    /// Desugars an assigment.
    /// Note that this converts the assignment's `ASTPattern` into a `CSTPattern`
    pub fn assign(&mut self, p: Spanned<ASTPattern>, e: Spanned<AST>) -> Result<CST, Syntax> {
        let p_span = p.span.clone();

        Ok(CST::assign(
            p.map(CSTPattern::try_from)
                .map_err(|err| Syntax::error(&err, &p_span))?,
            self.walk(e)?
        ))
    }

    /// Desugars a lambda
    /// This converts both patterns and expressions;
    /// On top of this, it desugars `a b c -> d`
    /// into `a -> b -> c -> d`.
    pub fn lambda(&mut self, p: Spanned<ASTPattern>, e: Spanned<AST>) -> Result<CST, Syntax> {
        let p_span = p.span.clone();
        let arguments = if let ASTPattern::Chain(c) = p.item { c } else { vec![p] };
        let mut expression = self.walk(e)?;

        for argument in arguments.into_iter().rev() {
            let pattern = argument.map(CSTPattern::try_from)
                .map_err(|err| Syntax::error(&err, &p_span))?;

            let combined = Span::combine(&pattern.span, &expression.span);
            expression   = Spanned::new(CST::lambda(pattern, expression), combined);
        }

        return Ok(expression.item);
    }

    /// Desugars a macro definition.
    /// Right now, this is a bit awkward;
    /// Ideally, a preprocessing step should be taken
    /// That determines which variables are declared where,
    /// Which macros are declared when,
    /// And removes all such valueless declarations from the AST.
    pub fn rule(&mut self, arg_pat: Spanned<ArgPattern>, tree: Spanned<AST>) -> Result<CST, Syntax> {
        let patterns_span = arg_pat.span.clone();
        let rule = Rule::new(arg_pat, tree)?;
        self.rules.push(Spanned::new(rule, patterns_span));

        // TODO: do a pre-pass where macros are scoped and removed?
        // TODO: return nothing?
        Ok(CST::Block(vec![]))
    }
}
