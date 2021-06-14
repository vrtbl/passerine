use std::{
    convert::TryFrom,
    collections::HashMap,
};

use crate::common::span::{Span, Spanned};
use crate::compiler::syntax::Syntax;
use crate::construct::{
    tree::{AST, Pattern, ArgPattern},
    symbol::SharedSymbol,
};

// TODO: immutably capture external values used by macro
// TODO: add context for macro application
// NOTE: add spans?

// TODO: make a different struct for applying macros
// i.e. a rule is like a template, but we need to instantiate it to expand
// bindings, mangles, and lowest_shared would be in this struct

/// When a macro is expanded, `AST` slices captured by the macro Argument Pattern
/// are spliced into the macro body.
/// A `Binding` relates a name (within an Argument Pattern),
/// to an `AST` slice.
type Bindings = HashMap<SharedSymbol, Spanned<AST>>;

/// Maps mangled symbols to unmangles symbols
type Mangles  = HashMap<SharedSymbol, SharedSymbol>;

/// A rule has an Argument Pattern and an `AST`.
/// When a form matches the `ArgPattern`,
/// a set of bindings are produced,
/// which are then spliced into the Rule's `AST`
/// to make a new `AST`.
/// This is done in a hygenic manner.
#[derive(Debug, Clone)]
pub struct Rule {
    pub arg_pat: Spanned<ArgPattern>,
    pub tree:    Spanned<AST>,
}

impl Rule {
    /// Builds a new rule, making sure the rule's signature is valid.
    pub fn new(
        arg_pat: Spanned<ArgPattern>,
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

    /// Returns all keywords, as a `(SharedSymbol, String) `tuple,
    /// used by the macro, in order of usage.
    /// Does not filter for duplicates.
    pub fn keywords(arg_pat: &Spanned<ArgPattern>) -> Vec<(SharedSymbol, String)> {
        match &arg_pat.item {
            ArgPattern::Group(pats) => {
                let mut keywords = vec![];
                for pat in pats { keywords.append(&mut Rule::keywords(&pat)) }
                keywords
            },
            ArgPattern::Keyword(name) => vec![(*name, arg_pat.span.contents())],
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

    // TODO: should we remove *all* layers?
    /// Takes a mangled shared symbol, and demangles it to one that
    /// is guaranteed to have been user-defined.
    pub fn remove_tag(name: &SharedSymbol, mangles: &Mangles) -> SharedSymbol {
        print!("lookup! {:?}", name);
        let mut result = *name;
        while let Some(removed) = mangles.get(&result) {
            result = *removed
        }
        println!(" -> is {:?}", result);
        return result;
    }

    // TODO: just use a UniqueSymbol?
    /// Creates a new unique shared symbol, to ensure the macro is hygienic
    pub fn unique_tag(
        name:          SharedSymbol,
        lowest_shared: &mut usize,
        mangles:       &mut Mangles,
    ) -> SharedSymbol {
        println!("replace! {:?}", name);
        *lowest_shared += 1;
        let new = SharedSymbol(*lowest_shared);
        mangles.insert(new, name);
        println!("{:#?}", mangles);
        return new;
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
    pub fn bind(
        arg_pat:           &Spanned<ArgPattern>,
        mut reversed_form: &mut Vec<Spanned<AST>>,
        lowest_shared:     &mut usize,
        mangles:           &mut Mangles,
    ) -> Option<Result<Bindings, Syntax>> {
        match &arg_pat.item {
            // TODO: right now, if a macro is invoked from another macro,
            // passerine won't recognize it,
            // because the pseudokeywords are hygenically replaced.
            // this should return true if a substituted pseudokeword
            // matches as well.
            // substitution scheme could be: `#name#tag`
            // and if name matches whole symbol matches.
            ArgPattern::Keyword(expected) => match reversed_form.pop()?.item {
                AST::Symbol(name) if &Rule::remove_tag(&name, &mangles) == expected => {
                    Some(Ok(HashMap::new()))
                },
                AST::Symbol(name) => {
                    println!("no dice {:?} -> {:?} = {:?}",
                        name,
                        &Rule::remove_tag(&name, &mangles),
                        expected);
                    None
                },
                _ => None,
            },
            ArgPattern::Symbol(symbol) => Some(Ok(
                vec![(symbol.clone(), reversed_form.pop()?)]
                    .into_iter().collect()
            )),
            ArgPattern::Group(pats) => {
                let mut bindings = HashMap::new();
                for pat in pats {
                    let span = pat.span.clone();
                    let new = match Rule::bind(&pat, &mut reversed_form, lowest_shared, mangles)? {
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

    /// Resolves a symbol.
    /// If the symbol has been bound, i.e. is defined in the Argument CSTPattern,
    /// we simply splice that in.
    /// If not, we hygenically replace it with a unique variable.
    pub fn resolve_symbol(
        name:          SharedSymbol,
        span:          Span,
        bindings:      &mut Bindings,
        lowest_shared: &mut usize,
        mangles:       &mut Mangles,
    ) -> Spanned<AST> {
        if let Some(bound_tree) = bindings.get(&name) {
            bound_tree.clone()
        } else {
            let unique = Rule::unique_tag(name, lowest_shared, mangles);
            let spanned = Spanned::new(AST::Symbol(unique), span.clone());
            bindings.insert(name, spanned);
            Spanned::new(AST::Symbol(unique), span)
        }
    }

    // TODO: move expansions to ast?

    /// Expands the bindings in a pattern.
    pub fn expand_pattern(
        pattern: Spanned<ASTPattern>,
        bindings: &mut Bindings,
        lowest_shared: &mut usize,
        mangles:       &mut Mangles,
    ) -> Result<Spanned<ASTPattern>, Syntax> {
        Ok(
            match pattern.item {
                ASTPattern::Symbol(name) => {
                    let span = pattern.span.clone();

                    Rule::resolve_symbol(name, pattern.span, bindings, lowest_shared, mangles)
                    .map(ASTPattern::try_from)
                    .map_err(|s| Syntax::error(&s, &span))?
                },
                ASTPattern::Data(_) => pattern,
                // TODO: treat name as symbol?
                ASTPattern::Label(name, pattern) => {
                    let span = pattern.span.clone();
                    Spanned::new(
                        ASTPattern::label(name, Rule::expand_pattern(*pattern, bindings, lowest_shared, mangles)?), span,
                    )
                },
                ASTPattern::Chain(chain) => {
                    let span = Spanned::build(&chain);
                    let expanded = chain.into_iter()
                        .map(|b| Rule::expand_pattern(b, bindings, lowest_shared, mangles))
                        .collect::<Result<Vec<_>, _>>()?;
                    Spanned::new(ASTPattern::Chain(expanded), span)
                },
                ASTPattern::Tuple(tuple) => {
                    let span = Spanned::build(&tuple);
                    let expanded = tuple.into_iter()
                        .map(|b| Rule::expand_pattern(b, bindings, lowest_shared, mangles))
                        .collect::<Result<Vec<_>, _>>()?;
                    Spanned::new(ASTPattern::Tuple(expanded), span)
                }
            }
        )
    }

    /// ~Macros inside of macros is a bit too meta for me to think about atm.~
    /// No longer!
    /// A macro inside a macro is a macro completely local to that macro.
    /// The argument patterns inside a macro can be extended.
    pub fn expand_arg_pat(
        arg_pat: Spanned<ArgPattern>,
        bindings: &mut Bindings,
        lowest_shared: &mut usize,
        mangles:       &mut Mangles,
    ) -> Result<Spanned<ArgPattern>, Syntax> {
        Ok(
            match arg_pat.item {
                ArgPattern::Keyword(_) => arg_pat,
                ArgPattern::Symbol(name) => {
                    let span = arg_pat.span.clone();

                    Rule::resolve_symbol(name, arg_pat.span, bindings, lowest_shared, mangles)
                    .map(ArgPattern::try_from)
                    .map_err(|s| Syntax::error(&s, &span))?
                },
                ArgPattern::Group(sub_pat) => {
                    let span = Spanned::build(&sub_pat);
                    let expanded = sub_pat.into_iter()
                        .map(|b| Rule::expand_arg_pat(b, bindings, lowest_shared, mangles))
                        .collect::<Result<Vec<_>, _>>()?;
                    Spanned::new(ArgPattern::Group(expanded), span)
                },
            }
        )
    }

    // TODO: break expand out into functions

    /// Takes a macro's tree and a set of bindings and produces a new hygenic tree.
    pub fn expand(
        tree: Spanned<AST>,
        mut bindings: &mut Bindings,
        lowest_shared: &mut usize,
        mangles:       &mut Mangles,
    ) -> Result<Spanned<AST>, Syntax> {
        // TODO: should macros evaluate arguments as thunks before insertions?
        // TODO: allow macros to reference external definitions
        let item: AST = match tree.item {
            // looks up symbol name in table of bindings
            // if it's found, it's replaced -
            // if it's not found, it's added to the table of bindings,
            // and replaced with a random symbol that does not collide with any other bindings
            // so that the next time the symbol is located,
            // it's consistently replaced, hygenically.
            AST::Symbol(name) => return Ok(Rule::resolve_symbol(
                name, tree.span.clone(), &mut bindings, lowest_shared, mangles
            )),

            AST::Data(_) => return Ok(tree),

            // Apply the transformation to each form
            AST::Block(forms) => AST::Block(
                forms.into_iter()
                    .map(|f| Rule::expand(f, bindings, lowest_shared, mangles))
                    .collect::<Result<Vec<_>, _>>()?
            ),

            // Apply the transformation to each item in the form
            AST::Form(branches) => AST::Form(
                branches.into_iter()
                    .map(|b| Rule::expand(b, bindings, lowest_shared, mangles))
                    .collect::<Result<Vec<_>, _>>()?
            ),

            AST::Group(expression) => AST::group(Rule::expand(*expression, bindings, lowest_shared, mangles)?),

            // Appy the transformation to the left and right sides of the comp
            AST::Comp { argument, function } => {
                let a = Rule::expand(*argument, bindings, lowest_shared, mangles)?;
                let f = Rule::expand(*function, bindings, lowest_shared, mangles)?;
                AST::comp(a, f)
            },

            // replace the variables in (argument) patterns
            AST::Pattern(pattern) => {
                let spanned = Spanned::new(pattern, tree.span.clone());
                AST::Pattern(Rule::expand_pattern(spanned, bindings, lowest_shared, mangles)?.item)
            },
            AST::ArgPattern(arg_pat) => {
                let spanned = Spanned::new(arg_pat, tree.span.clone());
                AST::ArgPattern(Rule::expand_arg_pat(spanned, bindings, lowest_shared, mangles)?.item)
            },

            // replace the variables in the patterns and the expression
            AST::Assign { pattern, expression } => {
                let p = Rule::expand_pattern(*pattern, bindings, lowest_shared, mangles)?;
                let e = Rule::expand(*expression, bindings, lowest_shared, mangles)?;
                AST::assign(p, e)
            },
            AST::Lambda { pattern, expression } => {
                let p = Rule::expand_pattern(*pattern, bindings, lowest_shared, mangles)?;
                let e = Rule::expand(*expression, bindings, lowest_shared, mangles)?;
                AST::lambda(p, e)
            },

            // TODO: Should labels be bindable in macros?
            AST::Label(kind, expression) => AST::Label(
                kind, Box::new(Rule::expand(*expression, bindings, lowest_shared, mangles)?)
            ),

            AST::Tuple(tuple) => AST::Tuple(
                tuple.into_iter()
                    .map(|b| Rule::expand(b, bindings, lowest_shared, mangles))
                    .collect::<Result<Vec<_>, _>>()?
            ),

            // a macro inside a macro. not sure how this should work yet
            AST::Syntax { arg_pat, expression } => {
                let ap = Rule::expand_arg_pat(*arg_pat, bindings, lowest_shared, mangles)?;
                let e = Rule::expand(*expression, bindings, lowest_shared, mangles)?;
                AST::syntax(ap, e);
                return Err(Syntax::error(
                    "Nested macros are not allowed at the moment",
                    &tree.span,
                ))?;
            },

            AST::FFI { name, expression } => AST::ffi(
                &name,
                Rule::expand(*expression, bindings, lowest_shared, mangles)?
            ),

            AST::Record(record) => AST::Record(
                record.into_iter()
                    .map(|b| Rule::expand(b, bindings, lowest_shared, mangles))
                    .collect::<Result<Vec<_>, _>>()?
            ),

            AST::Is { field, expression } => AST::is(
                Rule::expand(*field,      bindings, lowest_shared, mangles)?,
                Rule::expand(*expression, bindings, lowest_shared, mangles)?,
            )
        };

        return Ok(Spanned::new(item, tree.span));
    }
}
