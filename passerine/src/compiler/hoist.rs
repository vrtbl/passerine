use std::collections::HashMap;

use crate::{
    common::span::{Span, Spanned},
    compiler::syntax::{Note, Syntax},
    construct::{
        scope::Scope,
        symbol::{SharedSymbol, SymbolTable, UniqueSymbol},
        tree::{Base, Lambda, Pattern, ScopedLambda, CST, SST},
    },
};

// TODO: hoisting before expansion??
// TODO: hoist labels? how are labels declared? types?

// TODO: add a hoisting method to finalize hoists.

// TODO: keep track of syntax, do hoisting before macro expansion??

// TLDR: shouldn't the scope for types and macros be determined?

// NOTE: two goals:
// 1. replace all same-reference symbols with a unique
// integer 2. Build up a table of which symbols are
// accessible in what scopes.

/// Keeps track of:
/// 1. Local and nonlocal variables in each scope.
/// 2. All variables declared.
/// 3. Variables that have been used but not declared.
pub struct Hoister {
    /// The unique local symbols in the current scope.
    scopes: Vec<Scope>,
    /// Maps integers (index in vector) to string
    /// representation of symbol.
    symbol_table: SymbolTable,
    /// Keeps track of variables that were referenced before
    /// assignment.
    unresolved_hoists: HashMap<SharedSymbol, Spanned<UniqueSymbol>>,
}

impl Hoister {
    /// Creates a new hoisted in a root scope.
    /// Note that the hoister will always have a root scope.
    fn new() -> Hoister {
        Hoister {
            scopes: vec![Scope::new()],
            symbol_table: SymbolTable::new(),
            unresolved_hoists: HashMap::new(),
        }
    }

    /// Simple function that a scoped syntax tree (`SST`)
    /// from an `CST`. Replaces all symbols with unique
    /// identifiers; symbols by the same name in
    /// different scopes will get different identifiers.
    /// Also resolves closure captures and closure hoisting.
    pub fn hoist(
        tree: Spanned<CST>,
        symbols: HashMap<String, SharedSymbol>,
    ) -> Result<(Spanned<SST>, Scope), Syntax> {
        let mut hoister = Hoister::new();

        let sst = hoister.walk(tree)?;
        let scope = hoister.scopes.pop().unwrap();

        if !hoister.unresolved_hoists.is_empty() {
            let num_unresolved = hoister.unresolved_hoists.len();

            let mut error = Syntax::error_no_note(&format!(
                "{} variable{} referenced before assignment",
                num_unresolved,
                if num_unresolved == 1 { "" } else { "s" }
            ));

            // TODO: sort by occurence, earliest first?
            for (_symbol, spanned) in hoister.unresolved_hoists.iter() {
                // TODO: hints to correct to similar names, etc.
                // dbg!(&spanned.span);
                error = error.add_note(Note::new(spanned.span.clone()));
            }

            Err(error)
        } else {
            Ok((sst, scope))
        }
    }

    /// Enters a new scope, called when entering a new
    /// function.
    fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }
    /// Enters an existing scope, called when resolving
    /// variables.
    fn reenter_scope(&mut self, scope: Scope) {
        self.scopes.push(scope)
    }

    /// Exits the current scope, returning it.
    /// If there are no enclosing scopes, returns `None`.
    fn exit_scope(&mut self) -> Option<Scope> {
        if self.scopes.len() == 1 {
            return None;
        }
        if let Some(mut scope) = self.scopes.pop() {
            for local in scope.locals.items() {
                let name = self.symbol_table.name(&local);
                if self.unresolved_hoists.contains_key(&name) {
                    scope.locals.remove(&local);
                }
            }
            dbg!(&scope);
            dbg!(&self.unresolved_hoists);
            Some(scope)
        } else {
            unreachable!("no scopes left on stack?");
            None
        }
    }

    /// Returns the topmost, i.e. local, scope, mutably.
    fn local_scope(&mut self) -> &mut Scope {
        let last = self.scopes.len() - 1;
        &mut self.scopes[last]
    }

    /// Returns the topmost scope immutably.
    fn borrow_local_scope(&self) -> &Scope {
        let last = self.scopes.len() - 1;
        &self.scopes[last]
    }

    /// Walks a `CST` to produce an `SST`.
    /// This is fairly standard - hoisting happens in
    /// `self.assign`, `self.lambda`, and `self.symbol`.
    fn walk(&mut self, tree: Spanned<CST>) -> Result<Spanned<SST>, Syntax> {
        let sst: SST = match tree.item {
            CST::Base(Base::Lit(data)) => SST::Base(Base::Lit(data)),
            CST::Base(Base::Symbol(name)) => self.symbol(name, tree.span.clone()),
            CST::Base(Base::Block(block)) => self.block(block)?,
            // TODO: hoist as well
            CST::Base(Base::Label(name)) => {
                todo!()
                // SST::label(
                //     // TODO: change this to the following lines after types:
                //     self.resolve_symbol(name, cst.span.clone()),
                // )
            }
            CST::Base(Base::Tuple(tuple)) => self.tuple(tuple)?,
            CST::Base(Base::Assign(pattern, expression)) => self.assign(pattern, *expression)?,
            CST::Lambda(Lambda { arg, body }) => self.lambda(arg, *body)?,
            CST::Base(Base::Call(fun, arg)) => self.call(*fun, *arg)?,
            CST::Base(Base::Module(_)) => todo!(),
            CST::Base(Base::Effect(_)) => todo!(),
        };

        return Ok(Spanned::new(sst, tree.span));
    }

    /// Walks a pattern. If `declare` is true, we shadow
    /// variables in existing scopes and creates a new
    /// variable in the local scope.
    fn walk_pattern(
        &mut self,
        pattern: Spanned<Pattern<SharedSymbol>>,
        declare: bool,
    ) -> Spanned<Pattern<UniqueSymbol>> {
        let item = match pattern.item {
            Pattern::Symbol(name) => Pattern::Symbol(self.resolve_assign(name, declare)),
            Pattern::Lit(l) => Pattern::Lit(l),
            Pattern::Label(n, p) => Pattern::Label(
                // TODO: This is temoprary. Makes first use the definition.
                // Once we have types, change the following line to:
                // self.resolve_symbol(n, pattern.span.clone()),
                // until then, this will do:
                Spanned::new(self.resolve_assign(n.item, false), n.span),
                Box::new(self.walk_pattern(*p, declare)),
            ),
            Pattern::Tuple(t) => Pattern::Tuple(
                t.into_iter()
                    .map(|c| self.walk_pattern(c, declare))
                    .collect::<Vec<_>>(),
            ),
            Pattern::Chain(_) => todo!("Chained Patterns not yet implemented"),
        };

        return Spanned::new(item, pattern.span);
    }

    // TODO: local_symbol and nonlocal_symbol are so bad, I can't even.

    /// Looks to see whether a name is defined as a local in
    /// the current scope.
    fn local_symbol(&self, name: SharedSymbol) -> Option<UniqueSymbol> {
        for local in self.borrow_local_scope().locals.items().iter() {
            let local_name = self.symbol_table.name(local);
            if local_name == name {
                return Some(*local);
            }
        }

        return None;
    }

    /// Looks to see whether a name is used as a nonlocal in
    /// the current scope.
    fn nonlocal_symbol(&self, name: SharedSymbol) -> Option<UniqueSymbol> {
        for nonlocal in self.borrow_local_scope().nonlocals.items().iter() {
            let nonlocal_name = self.symbol_table.name(nonlocal);
            if nonlocal_name == name {
                return Some(*nonlocal);
            }
        }

        return None;
    }

    /// Adds a symbol as a captured variable in all scopes.
    /// Used in conjunction with `uncapture_all` to build
    /// hoisting chains.
    fn capture_all(&mut self, unique_symbol: UniqueSymbol) {
        for scope in self.scopes.iter_mut() {
            scope.nonlocals.push(unique_symbol);
        }
    }

    /// Removes a symbol as a captured variable in all
    /// scopes. This ensures that the hoisting chain
    /// only goes back to the most recent declaration.
    fn uncapture_all(&mut self, unique_symbol: UniqueSymbol) {
        for scope in self.scopes.iter_mut() {
            scope.nonlocals.remove(&unique_symbol);
        }
    }

    /// Tries to resolve a variable lookup:
    /// 1. If this variable is local or nonlocal to this
    /// scope, use it. 2. If this variable is defined in
    /// an enclosing scope, capture it and use it. 3. If
    /// this variable is not defined, return `None`.
    fn try_resolve(&mut self, name: SharedSymbol) -> Option<UniqueSymbol> {
        if let Some(unique_symbol) = self.local_symbol(name) {
            return Some(unique_symbol);
        }
        if let Some(unique_symbol) = self.nonlocal_symbol(name) {
            return Some(unique_symbol);
        }

        if let Some(scope) = self.exit_scope() {
            let resolved = self.try_resolve(name);
            self.reenter_scope(scope);
            if let Some(unique_symbol) = resolved {
                self.local_scope().nonlocals.push(unique_symbol);
                return Some(unique_symbol);
            }
        }

        // variable is not defined in this or enclosing scopes
        return None;
    }

    /// Returns the unique usize of a local symbol.
    /// If a variable is referenced before assignment,
    /// this function will define it in all lexical scopes
    /// once this variable is discovered, we remove the
    /// definitions in all scopes below this one.
    fn resolve_assign(&mut self, name: SharedSymbol, redeclare: bool) -> UniqueSymbol {
        // if we've seen the symbol before but don't know where it's
        // defined
        if let Some(unique_symbol) = self.unresolved_hoists.get(&name) {
            if self.local_symbol(name).is_none() {
                // dbg!(self.nonlocal_symbol(name));
                // dbg!(&name);
                // dbg!(&self.symbol_table);
                // dbg!(&self.scopes);
                // // dbg!(&self.try_resolve(name));
                // // panic!();
                // // this is a definition; we've resolved it!
                let unique_symbol = unique_symbol.item;
                self.uncapture_all(unique_symbol);
                self.unresolved_hoists.remove(&name);
                self.local_scope().locals.push(unique_symbol);
                return unique_symbol;
            }
        }

        // if we haven't seen the symbol before,
        // we search backwards through scopes and build a hoisting
        // chain
        if !redeclare {
            if let Some(unique_symbol) = self.try_resolve(name) {
                return unique_symbol;
            }
        }

        // if we didn't find it by searching backwards, we declare
        // it in the current scope
        let unique_symbol = self.symbol_table.push(name);
        self.local_scope().locals.push(unique_symbol);
        return unique_symbol;
    }

    /// This function wraps try_resolve,
    /// but checks that the symbol is unresolved first.
    fn resolve_symbol(&mut self, name: SharedSymbol, span: Span) -> UniqueSymbol {
        // if we've seen the symbol before but don't know where it's
        // defined
        if let Some(unique_symbol) = self.unresolved_hoists.get(&name) {
            return unique_symbol.item;
        }

        // if we haven't seen the symbol before,
        // we search backwards through scopes and build a hoisting
        // chain
        if let Some(unique_symbol) = self.try_resolve(name) {
            return unique_symbol;
        }

        // if we didn't find it by searching backwards, we mark it
        // as unresolved
        let unique_symbol = self.symbol_table.push(name);
        self.capture_all(unique_symbol);
        self.unresolved_hoists
            .insert(name, Spanned::new(unique_symbol, span));

        // put it in the local scope so we can check for use before
        self.local_scope().locals.push(unique_symbol);
        return unique_symbol;
    }

    /// Replaces a symbol name with a unique identifier for
    /// that symbol
    fn symbol(&mut self, name: SharedSymbol, span: Span) -> SST {
        // if we are hoisting the variable,
        // mark the variable as being used before its lexical
        // definition
        return SST::Base(Base::Symbol(self.resolve_symbol(name, span)));
    }

    /// Walks a block, nothing fancy here.
    fn block(&mut self, block: Vec<Spanned<CST>>) -> Result<SST, Syntax> {
        let mut expressions = vec![];
        for expression in block {
            expressions.push(self.walk(expression)?)
        }

        Ok(SST::Base(Base::Block(expressions)))
    }

    /// Walks a tuple, nothing fancy here.
    fn tuple(&mut self, tuple: Vec<Spanned<CST>>) -> Result<SST, Syntax> {
        let mut expressions = vec![];
        for expression in tuple {
            expressions.push(self.walk(expression)?)
        }

        Ok(SST::Base(Base::Tuple(expressions)))
    }

    /// Walks an assignment.
    /// Delegates to `walk_pattern` for capturing.
    /// Assignments can capture existing variables
    fn assign(
        &mut self,
        pattern: Spanned<Pattern<SharedSymbol>>,
        expression: Spanned<CST>,
    ) -> Result<SST, Syntax> {
        let sst_pattern = self.walk_pattern(pattern, false);
        let sst_expression = self.walk(expression)?;

        return Ok(SST::Base(Base::assign(sst_pattern, sst_expression)));
    }

    /// Walks a function definition.
    /// Like `assign`, delegates to `walk_pattern` for
    /// capturing. But any paramaters will shadow those
    /// in outer scopes.
    fn lambda(
        &mut self,
        pattern: Spanned<Pattern<SharedSymbol>>,
        expression: Spanned<CST>,
    ) -> Result<SST, Syntax> {
        self.enter_scope();
        let arg = self.walk_pattern(pattern, true);
        let body = Box::new(self.walk(expression)?);
        let scope = self.exit_scope().unwrap();

        return Ok(SST::ScopedLambda(ScopedLambda { arg, body, scope }));
    }

    /// Walks a function call.
    fn call(&mut self, fun: Spanned<CST>, arg: Spanned<CST>) -> Result<SST, Syntax> {
        return Ok(SST::Base(Base::call(self.walk(fun)?, self.walk(arg)?)));
    }
}

#[cfg(test)]
mod test_super {
    use std::ops::Not;

    use super::*;
    use crate::{
        common::Source,
        compiler::{Desugarer, Lexer, Parser, Reader},
    };

    fn test_source(source: &str) -> bool {
        let tokens = Lexer::lex(Source::source(source)).unwrap();
        let token_tree = Reader::read(tokens).unwrap();
        let (ast, symbols) = Parser::parse(token_tree).unwrap();
        let cst = Desugarer::desugar(ast);
        let result = Hoister::hoist(cst, symbols);
        dbg!(&result);
        return result.is_ok();
    }

    #[test]
    fn use_before() {
        assert!(test_source("x; x = 0").not());
    }

    #[test]
    fn use_capture_after() {
        assert!(test_source("() -> x; x = 7"));
    }

    #[test]
    fn use_before_capture_after() {
        assert!(test_source("x; () -> x; x = 0").not());
    }

    #[test]
    fn nested_capture() {
        assert!(test_source("_ -> { x = _ -> pi; pi = 3 }; pi = 3.14"));
    }
}
