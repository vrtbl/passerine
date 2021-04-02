use std::collections::HashMap;

use crate::common::span::Spanned;
use crate::compiler::{
    cst::{CST, CSTPattern},
    sst::{SST, SSTPattern, UniqueSymbol, Scope},
    syntax::Syntax,
};

// TODO: hoisting before expansion.
// TODO: hoist labels? how are labels declared? types?
// TODO: once modules exist, the entire program should be wrapped in a module.
// TODO: add a hoisting method to finalize hoists.

// NOTE: two goals:
// 1. replace all same-reference symbols with a unique integer
// 2. Build up a table of which symbols are accessible in what scopes.

/// Simple function that a scoped syntax tree (`SST`) from an `CST`.
/// Replaces all symbols with unique identifiers;
/// symbols by the same name in different scopes will get different identifiers.
/// Also resolves closure captures and closure hoisting.
pub fn hoist(cst: Spanned<CST>) -> Result<(Spanned<SST>, Scope), Syntax> {
    let mut hoister = Hoister::new();
    let sst = hoister.walk(cst)?;
    let scope = hoister.scopes.pop().unwrap();

    if !hoister.unresolved_hoists.is_empty() {
        // TODO: Actual errors
        return Err(Syntax::error(
            &format!(
                "{} were referenced before assignment",
                hoister.unresolved_hoists.keys()
                    .map(|s| format!("'{}'", s))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            &sst.span,
        ))
    }

    return Ok((sst, scope));
}

/// Keeps track of:
/// 1. Local and nonlocal variables in each scope.
/// 2. All variables declared.
/// 3. Variables that have been used but not declared.
pub struct Hoister {
    /// The unique local symbols in the current scope.
    scopes: Vec<Scope>,
    // TODO: make it it's own type
    /// Maps integers (index in vector) to string representation of symbol.
    symbol_table: Vec<String>,
    /// Keeps track of variables that were referenced before assignment.
    unresolved_hoists: HashMap<String, UniqueSymbol>,
}

impl Hoister {
    /// Creates a new hoisted in a root scope.
    /// Note that the hoister will always have a root scope.
    pub fn new() -> Hoister {
        Hoister {
            scopes:            vec![Scope::new()],
            symbol_table:      vec![],
            unresolved_hoists: HashMap::new(),
        }
    }

    /// Enters a new scope, called when entering a new function.
    fn   enter_scope(&mut self) { self.scopes.push(Scope::new()); }
    /// Enters an existing scope, called when resolving variables.
    fn reenter_scope(&mut self, scope: Scope) { self.scopes.push(scope) }

    /// Exits the current scope, returning it.
    /// If there are no enclosing scopes, returns `None`.
    fn exit_scope(&mut self) -> Option<Scope> {
         if self.scopes.len() == 1 { return None; }
        return self.scopes.pop()
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
    ///`self.assign`, `self.lambda`, and `self.symbol`.
    pub fn walk(&mut self, cst: Spanned<CST>) -> Result<Spanned<SST>, Syntax> {
        let sst: SST = match cst.item {
            CST::Data(data) => SST::Data(data),
            CST::Symbol(name) => self.symbol(&name),
            CST::Block(block) => self.block(block)?,
            CST::Label(name, expression) => SST::Label(name, Box::new(self.walk(*expression)?)),
            CST::Tuple(tuple) => self.tuple(tuple)?,
            CST::FFI    { name,    expression } => SST::ffi(&name, self.walk(*expression)?),
            CST::Assign { pattern, expression } => self.assign(*pattern, *expression)?,
            CST::Lambda { pattern, expression } => self.lambda(*pattern, *expression)?,
            CST::Call   { fun,     arg        } => self.call(*fun, *arg)?,
        };

        return Ok(Spanned::new(sst, cst.span))
    }

    /// Walks a pattern. If `declare` is true, we shadow variables in existing scopes
    /// and creates a new variable in the local scope.
    pub fn walk_pattern(&mut self, pattern: Spanned<CSTPattern>, declare: bool) -> Spanned<SSTPattern> {
        let item = match pattern.item {
            CSTPattern::Symbol(name) => {
                SSTPattern::Symbol(self.resolve_assign(&name, declare))
            },
            CSTPattern::Data(d)     => SSTPattern::Data(d),
            CSTPattern::Label(n, p) => SSTPattern::Label(n, Box::new(self.walk_pattern(*p, declare))),
            CSTPattern::Tuple(t)    => SSTPattern::Tuple(
                t.into_iter().map(|c| self.walk_pattern(c, declare)).collect::<Vec<_>>()
            )
        };

        return Spanned::new(item, pattern.span);
    }

    /// Creates a new symbol that is guaranteed to be unique.
    fn new_symbol(&mut self, name: &str) -> UniqueSymbol {
        let index = self.symbol_table.len();
        self.symbol_table.push(name.to_string());
        return UniqueSymbol(index);
    }

    /// Looks to see whether a name is defined as a local in the current scope.
    fn local_symbol(&self, name: &str) -> Option<UniqueSymbol> {
        for local in self.borrow_local_scope().locals.iter() {
            let local_name = &self.symbol_table[local.0];
            if local_name == name { return Some(*local); }
        }

        return None;
    }

    /// Looks to see whether a name is used as a nonlocal in the current scope.
    fn nonlocal_symbol(&self, name: &str) -> Option<UniqueSymbol> {
        for nonlocal in self.borrow_local_scope().nonlocals.iter() {
            let nonlocal_name = &self.symbol_table[nonlocal.0];
            if nonlocal_name == name { return Some(*nonlocal); }
        }

        return None;
    }

    /// Adds a symbol as a captured variable in all scopes.
    /// Used in conjunction with `uncapture_all` to build hoisting chains.
    fn capture_all(&mut self, unique_symbol: UniqueSymbol) {
        for scope in self.scopes.iter_mut() {
            scope.nonlocals.push(unique_symbol);
        }
    }

    /// Removes a symbol as a captured variable in all scopes.
    /// This ensures that the hoisting chain only goes back to the most recent declaration.
    fn uncapture_all(&mut self, unique_symbol: UniqueSymbol) {
        for scope in self.scopes.iter_mut() {
            // TODO: if let?
            let index = scope.nonlocal_index(unique_symbol).unwrap();
            scope.nonlocals.remove(index);
        }
    }

    /// Tries to resolve a variable lookup:
    /// 1. If this variable is local or nonlocal to this scope, use it.
    /// 2. If this variable is defined in an enclosing scope, capture it and use it.
    /// 3. If this variable is not defined, return `None`.
    fn try_resolve(&mut self, name: &str) -> Option<UniqueSymbol> {
        if let Some(unique_symbol) = self.local_symbol(name)    { return Some(unique_symbol); }
        if let Some(unique_symbol) = self.nonlocal_symbol(name) { return Some(unique_symbol); }

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

    // 1. If the variable is local to the scope, add it to `locals` in the current `Scope`
    // 2. If the variable is not local to the scope, search backwards through all enclosing scopes. If it is found and add it to the `nonlocals` of all the scopes we've searched through
    // 3. If the variable can not be found:
    //     a. If this is a declaration, i.e. `x = true` or `x -> {}` check `unresolved_captures` for `x`.
    //         i. If `x` is in unresolved captures, define it in the local scope, *remove* it from `unresolved_captures` and then *remove* it as captured in all enclosing scopes (a la rule 2).
    //     b. If this is an access, i.e `print x`, add `x` to a table called `unresolved_captures`, and mark `x` as captured a la rule 2 in all enclosing scopes.
    // 4. After compilation, if there are still variables in `unresolved_captures`, raise the error 'variable(s) referenced before assignment' (obviously point out which variables and where this occurred).

    /// Returns the unique usize of a local symbol.
    /// If a variable is referenced before assignment,
    /// this function will define it in all lexical scopes
    /// once this variable is discovered, we remove the definitions
    /// in all scopes below this one.
    fn resolve_assign(&mut self, name: &str, redeclare: bool) -> UniqueSymbol {
        // if we've seen the symbol before but don't know where it's defined
        if let Some(unique_symbol) = self.unresolved_hoists.get(name) {
            // this is a definition; we've resolved it!
            let unique_symbol = *unique_symbol;
            self.uncapture_all(unique_symbol);
            self.unresolved_hoists.remove(name);
            self.local_scope().locals.push(unique_symbol);
            return unique_symbol;
        }

        // if we haven't seen the symbol before,
        // we search backwards through scopes and build a hoisting chain
        if !redeclare {
            if let Some(unique_symbol) = self.try_resolve(name) { return unique_symbol; }
        }

        // if we didn't find it by searching backwards, we declare it in the current scope
        let unique_symbol = self.new_symbol(name);
        self.local_scope().locals.push(unique_symbol);
        return unique_symbol;
    }

    /// This function wraps try_resolve,
    /// but checks that the symbol is unresolved first.
    fn resolve_symbol(&mut self, name: &str) -> UniqueSymbol {
        // if we've seen the symbol before but don't know where it's defined
        if let Some(unique_symbol) = self.unresolved_hoists.get(name) {
            return *unique_symbol;
        }

        // if we haven't seen the symbol before,
        // we search backwards through scopes and build a hoisting chain
        if let Some(unique_symbol) = self.try_resolve(name) { return unique_symbol; }

        // if we didn't find it by searching backwards, we mark it as unresolved
        let unique_symbol = self.new_symbol(name);
        self.capture_all(unique_symbol);
        self.unresolved_hoists.insert(name.to_string(), unique_symbol);
        return unique_symbol;
    }

    /// Replaces a symbol name with a unique identifier for that symbol
    pub fn symbol(&mut self, name: &str) -> SST {
        // if we are hoisting the variable,
        // mark the variable as being used before its lexical definition
        return SST::Symbol(self.resolve_symbol(name));
    }

    /// Walks a block, nothing fancy here.
    pub fn block(&mut self, block: Vec<Spanned<CST>>) -> Result<SST, Syntax> {
        let mut expressions = vec![];
        for expression in block {
            expressions.push(self.walk(expression)?)
        }

        Ok(SST::Block(expressions))
    }

    /// Walks a tuple, nothing fancy here.
    pub fn tuple(&mut self, tuple: Vec<Spanned<CST>>) -> Result<SST, Syntax> {
        let mut expressions = vec![];
        for expression in tuple {
            expressions.push(self.walk(expression)?)
        }

        Ok(SST::Tuple(expressions))
    }

    /// Walks an assignment.
    /// Delegates to `walk_pattern` for capturing.
    /// Assignments can capture existing variables
    pub fn assign(&mut self, pattern: Spanned<CSTPattern>, expression: Spanned<CST>) -> Result<SST, Syntax> {
        let sst_pattern = self.walk_pattern(pattern, false);
        let sst_expression = self.walk(expression)?;

        return Ok(SST::assign(
            sst_pattern,
            sst_expression,
        ));
    }

    /// Walks a function definition.
    /// Like `assign`, delegates to `walk_pattern` for capturing.
    /// But any paramaters will shadow those in outer scopes.
    pub fn lambda(&mut self, pattern: Spanned<CSTPattern>, expression: Spanned<CST>) -> Result<SST, Syntax> {
        self.enter_scope();
        let sst_pattern = self.walk_pattern(pattern, true);
        let sst_expression = self.walk(expression)?;
        let scope = self.exit_scope().unwrap();

        return Ok(SST::lambda(
            sst_pattern,
            sst_expression,
            scope,
        ));
    }

    /// Walks a function call.
    pub fn call(&mut self, fun: Spanned<CST>, arg: Spanned<CST>) -> Result<SST, Syntax> {
        return Ok(SST::call(
            self.walk(fun)?,
            self.walk(arg)?,
        ));
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn messing_around() {
//         let result =
//     }
// }
