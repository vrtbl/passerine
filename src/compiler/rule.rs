use std::{
    convert::TryFrom,
    collections::HashMap,
};

use crate::common::{
    stamp::stamp,
    span::{Span, Spanned},
};

use crate::compiler::{
    ast::{AST, ASTPattern, ArgPat},
    syntax::Syntax
};

// TODO: immutably capture external values used by macro
// TODO: add context for macro application
// NOTE: add spans?

type Bindings = HashMap<String, Spanned<AST>>;

#[derive(Debug, Clone)]
pub struct Rule {
    pub arg_pat: Spanned<ArgPat>,
    pub tree: Spanned<AST>,
}

impl Rule {
    /// Builds a new rule, making sure the rule's signature is valid.
    pub fn new(
        arg_pat: Spanned<ArgPat>,
        tree: Spanned<AST>,
    ) -> Result<Rule, Syntax> {
        if Rule::keywords(&arg_pat).len() == 0 {
            return Err(Syntax::error(
                "Syntactic macro must have at least one pseudokeyword",
                &arg_pat.span,
            ));
        }
        Ok(Rule { arg_pat, tree })
    }

    /// Returns all keywords, as strings, used by the macro, in order of usage.
    /// Does not filter for duplicates.
    pub fn keywords(arg_pat: &Spanned<ArgPat>) -> Vec<String> {
        match &arg_pat.item {
            ArgPat::Group(pats) => {
                let mut keywords = vec![];
                for pat in pats { keywords.append(&mut Rule::keywords(&pat)) }
                keywords
            },
            ArgPat::Keyword(name) => vec![name.clone()],
            _ => vec![],
        }
    }

    /// Merges two maps of bindings.
    /// If there is a collision, i.e. a name bound in both bindings,
    /// An error highlighting the duplicate binding is returned.
    pub fn merge_safe(base: &mut Bindings, new: Bindings, def: Span) -> Result<(), Syntax> {
        let collision = Syntax::error(
            "Variable has already been declared in syntactic macro argument pattern", &def
        );

        for (n, t) in new {
            if base.contains_key(&n) { return Err(collision); }
            else                     { base.insert(n, t);     }
        }

        Ok(())
    }

    /// Traverses a form, creating bindings for subsequent transformation.
    /// Returns `None` if the form does not match the argument pattern.
    /// `Some(Ok(_))` if it matches successfully,
    /// and `Some(Err(_))` if it matches but something is incorrect.
    /// **You must check that the passed `&mut reversed_form` is empty
    /// to gaurantee the match occured in full**
    /// Note that this function takes the form unwrapped and in reverse -
    /// This is to make processing the bindings more efficient,
    /// As this function works with the head of the form.
    pub fn bind(arg_pat: &Spanned<ArgPat>, mut reversed_form: &mut Vec<Spanned<AST>>)
    -> Option<Result<Bindings, Syntax>> {
        match &arg_pat.item {
            ArgPat::Keyword(expected) => {
                if let AST::Symbol(name) = reversed_form.pop()?.item {
                    if &name == expected { Some(Ok(HashMap::new())) }
                    else                 { None                     }
                } else                   { None                     }
            },
            ArgPat::Symbol(symbol) => Some(Ok(
                vec![(symbol.clone(), reversed_form.pop()?)]
                    .into_iter().collect()
            )),
            ArgPat::Group(pats) => {
                let mut bindings = HashMap::new();
                for pat in pats {
                    let span = pat.span.clone();
                    let new = match Rule::bind(&pat, &mut reversed_form)? {
                        Ok(matched) => matched,
                        mismatch @ Err(_) => return Some(mismatch),
                    };
                    if let Err(collision) = Rule::merge_safe(&mut bindings, new, span) {
                        return Some(Err(collision));
                    }
                }
                Some(Ok(bindings))
            },
        }
    }

    /// Turns a base identifier into a random identifier
    /// of the format `#_<base>_XXXXXXXX`,
    /// Gauranteed not to exist in bindings.
    pub fn unique_identifier(base: String, bindings: &Bindings) -> String {
        let mut tries = 0;
        for _ in 0..1024 {
            let stamp = stamp(tries);
            // for example, `foo` may become `#_foo_d56aea12`
            // this should not be constructible as a symbol.
            let modified = format!("#_{}_{}", base, stamp);
            if !bindings.contains_key(&modified) {
                // println!("{}", modified);
                return modified;
            }
            tries += 1;
        }
        panic!("Generated 1024 new unique identifiers for macro expansion, but all were already in use!");
    }

    pub fn resolve_symbol(name: String, span: Span, bindings: &mut Bindings) -> Spanned<AST> {
        if let Some(bound_tree) = bindings.get(&name) {
            bound_tree.clone()
        } else {
            let unique = Rule::unique_identifier(name.clone(), bindings);
            let spanned = Spanned::new(AST::Symbol(unique.clone()), span.clone());
            bindings.insert(name, spanned);
            Spanned::new(AST::Symbol(unique), span)
        }
    }

    // TODO: move expansions to ast?

    pub fn expand_pattern(
        pattern: Spanned<ASTPattern>,
        bindings: &mut Bindings,
    ) -> Result<Spanned<ASTPattern>, Syntax> {
        Ok(
            match pattern.item {
                ASTPattern::Symbol(name) => Rule::resolve_symbol(name, pattern.span, bindings)
                    .map(ASTPattern::try_from).unwrap(),
                ASTPattern::Data(_) => pattern,
                // treat name as symbol?
                ASTPattern::Label(name, pattern) => {
                    let span = pattern.span.clone();
                    Spanned::new(
                        ASTPattern::label(name, Rule::expand_pattern(*pattern, bindings)?), span,
                    )
                },
                ASTPattern::Chain(_) => todo!(),
            }
        )
    }

    // Macros inside of macros is a bit too meta for me to think about atm.
    pub fn expand_arg_pat(
        _arg_pat: Spanned<ArgPat>,
        _bindings: &mut Bindings,
    ) -> Result<Spanned<ArgPat>, Syntax> {
        Err(Syntax::error(
            "Macros in macros are not yet implemented",
            &Span::empty(),
        ))
    }

    /// Takes a macro's tree and a set of bindings and produces a new hygenic tree.
    pub fn expand(tree: Spanned<AST>, mut bindings: &mut Bindings)
    -> Result<Spanned<AST>, Syntax> {
        // TODO: should macros evaluate arguments as thunks before insertions?
        // TODO: allow macros to reference external definitions
        let item: AST = match tree.item {
            // looks up symbol name in table of bindings
            // if it's found, it's replaced -
            // if it's not found, it's added to the table of bindings,
            // and replaced with a random symbol that does not collide with any other bindings
            // so that the next time the symbol is located,
            // it's consistently replaced, hygenically.
            AST::Symbol(name) => return Ok(Rule::resolve_symbol(name, tree.span.clone(), &mut bindings)),
            AST::Data(_) => return Ok(tree),

            // Apply the transformation to each form
            AST::Block(forms) => AST::Block(
                forms.into_iter()
                    .map(|f| Rule::expand(f, bindings))
                    .collect::<Result<Vec<_>, _>>()?
            ),

            // Apply the transformation to each item in the form
            AST::Form(branches) => AST::Form(
                branches.into_iter()
                    .map(|b| Rule::expand(b, bindings))
                    .collect::<Result<Vec<_>, _>>()?
            ),

            // replace the variables in (argument) patterns
            AST::Pattern(pattern) => {
                let spanned = Spanned::new(pattern, tree.span.clone());
                AST::Pattern(Rule::expand_pattern(spanned, bindings)?.item)
            },
            AST::ArgPat(arg_pat) => {
                let spanned = Spanned::new(arg_pat, tree.span.clone());
                AST::ArgPat(Rule::expand_arg_pat(spanned, bindings)?.item)
            },

            // replace the variables in the patterns and the expression
            AST::Assign { pattern, expression } => {
                let p = Rule::expand_pattern(*pattern, bindings)?;
                let e = Rule::expand(*expression, bindings)?;
                AST::assign(p, e)
            },
            AST::Lambda { pattern, expression } => {
                let p = Rule::expand_pattern(*pattern, bindings)?;
                let e = Rule::expand(*expression, bindings)?;
                AST::lambda(p, e)
            },

            AST::Print(expression) => AST::Print(
                Box::new(Rule::expand(*expression, bindings)?)
            ),

            // TODO: Should labels be bindable in macros?
            AST::Label(kind, expression) => AST::Label(
                kind, Box::new(Rule::expand(*expression, bindings)?)
            ),

            // a macro inside a macro. not sure how this should work yet
            AST::Syntax { arg_pat, expression } => {
                let ap = Rule::expand_arg_pat(*arg_pat, bindings)?;
                let e = Rule::expand(*expression, bindings)?;
                AST::syntax(ap, e)
            },
        };

        return Ok(Spanned::new(item, tree.span));
    }
}
