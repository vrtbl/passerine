use std::{
    mem,
    collections::HashMap,
};

use crate::common::span::{Span, Spanned};

use crate::compiler::{
    cst::{CST, CSTPattern},
    sst::{SST, SSTPattern, UniqueSymbol, Scope},
    syntax::Syntax,
};

// TODO: hoisting before expansion.
// TODO: hoist labels? how are labels declared? types?
// TODO: once modules exist, the entire program should be wrapped in a module.

// NOTE: two goals:
// 1. replace all same-reference symbols with a unique integer
// 2. Build up a table of which symbols are accessible in what scopes.

/// Simple function that a scoped syntax tree (`SST`) from an `CST`.
pub fn hoist(cst: Spanned<CST>) -> Result<(Spanned<SST>, Scope), Syntax> {
    let mut hoister = Hoister::new();
    let sst = hoister.walk(cst)?;
    let scope = hoister.scopes.pop().unwrap();
    let symbol_table = hoister.symbol_table;

    if !hoister.unresolved_hoists.is_empty() {
        // TODO: Actual errors
        return Err(Syntax::error(
            &format!(
                "Variable {} referenced before assignment",
                hoister.unresolved_hoists.iter().next().unwrap().0
            ),
            &Span::empty(),
        ))
    }

    return Ok((sst, scope, symbol_table));
}

pub struct Hoister {
    /// The unique local symbols in the current scope.
    scopes: Vec<Scope>,
    /// Maps integers (index in vector) to string representation of symbol.
    // TODO: make it it's own type
    symbol_table: Vec<String>,
    /// keeps track of variables that were referenced before assignment.
    unresolved_hoists: HashMap<String, UniqueSymbol>,
}

impl Hoister {
    pub fn new() -> Hoister {
        Hoister {
            scopes:            vec![Scope::new()],
            symbol_table:      vec![],
            unresolved_hoists: HashMap::new(),
        }
    }

    fn   enter_scope(&mut self)               { self.scopes.push(Scope::new()); }
    fn reenter_scope(&mut self, scope: Scope) { self.scopes.push(scope)         }

    fn exit_scope(&mut self) -> Option<Scope> {
         if self.scopes.len() == 1 { return None; }
        return self.scopes.pop()
    }

    fn local_scope(&mut self) -> &mut Scope {
        let last = self.scopes.len() - 1;
        &mut self.scopes[last]
    }

    fn borrow_local_scope(&self) -> &Scope {
        let last = self.scopes.len() - 1;
        &self.scopes[last]
    }

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

    pub fn walk_pattern(&mut self, pattern: Spanned<CSTPattern>, declare: bool) -> Spanned<SSTPattern> {
        let item = match pattern.item {
            CSTPattern::Symbol(name) => {
                SSTPattern::Symbol(self.resolve(&name, declare))
            },
            CSTPattern::Data(d)     => SSTPattern::Data(d),
            CSTPattern::Label(n, p) => SSTPattern::Label(n, Box::new(self.walk_pattern(*p, declare))),
            CSTPattern::Tuple(t)    => SSTPattern::Tuple(
                t.into_iter().map(|c| self.walk_pattern(c, declare)).collect::<Vec<_>>()
            )
        };

        return Spanned::new(item, pattern.span);
    }

    fn new_symbol(&mut self, name: &str) -> UniqueSymbol {
        let index = self.symbol_table.len();
        self.symbol_table.push(name.to_string());
        return UniqueSymbol(index);
    }

    fn local_symbol(&self, name: &str) -> Option<UniqueSymbol> {
        for local in self.borrow_local_scope().locals.iter() {
            let local_name = &self.symbol_table[local.0];
            if local_name == name { return Some(*local); }
        }

        return None;
    }

    fn nonlocal_symbol(&self, name: &str) -> Option<UniqueSymbol> {
        for nonlocal in self.borrow_local_scope().nonlocals.iter() {
            let nonlocal_name = &self.symbol_table[nonlocal.0];
            if nonlocal_name == name { return Some(*nonlocal); }
        }

        return None;
    }

    fn capture_all(&mut self, unique_symbol: UniqueSymbol) {
        for scope in self.scopes.iter_mut() {
            scope.nonlocals.push(unique_symbol);
        }
    }

    fn uncapture_all(&mut self, unique_symbol: UniqueSymbol) {
        for scope in self.scopes.iter_mut() {
            // TODO: if let?
            let index = scope.nonlocal_index(unique_symbol).unwrap();
            scope.nonlocals.remove(index);
        }
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
    fn resolve(&mut self, name: &str, declare: bool) -> Option<UniqueSymbol> {
        if let Some(unique_symbol) = self.unresolved_hoists.get(name) {

        }

        if let Some(unique_symbol) = self.local_symbol(name)    { return Some(unique_symbol); }
        if let Some(unique_symbol) = self.nonlocal_symbol(name) { return Some(unique_symbol); }

        if let Some(scope) = self.exit_scope() {
            if let Some(unique_symbol) = self.resolve(name, false) {
                self.local_scope().nonlocals.push(unique_symbol);
                self.reenter_scope(scope);
                return Some(unique_symbol);
            }
        }

        if declare

        let unique_symbol = self.new_symbol(name);
        self.unresolved_hoists.insert(name.to_string(), unique_symbol);
        return Some(unique_symbol);
    }

    fn resolve_symbol(&mut self, name: &str) -> UniqueSymbol {
        // if the symbol is defined in the enclosing scope,
        // return that symbol and marked it as being captured by the current scope
        if let Some(unique_symbol) = self.local_symbol(name)    { return unique_symbol; }
        if let Some(unique_symbol) = self.nonlocal_symbol(name) { return unique_symbol; }

        if let Some(scope) = self.exit_scope() {
            let unique_symbol = self.resolve_symbol(name);
            self.reenter_scope(scope);
            self.local_scope().nonlocals.push(unique_symbol);
            return unique_symbol;
        }

        // at this point there are no enclosing scopes and name has not been declared
        return match self.unresolved_hoists.get(name) {
            Some(unique_symbol) => *unique_symbol,
            None => {
                let unique_symbol = self.new_symbol(name);
                self.unresolved_hoists.insert(name.to_string(), unique_symbol);
                unique_symbol
            },
        };
    }

    /// Replaces a symbol name with a unique identifier for that symbol
    pub fn symbol(&mut self, name: &str) -> SST {
        // if we are hoisting the variable,
        // mark the variable as being used before its lexical definition
        return SST::Symbol(self.resolve_symbol(name));
    }

    pub fn block(&mut self, block: Vec<Spanned<CST>>) -> Result<SST, Syntax> {
        let mut expressions = vec![];
        for expression in block {
            expressions.push(self.walk(expression)?)
        }

        Ok(SST::Block(expressions))
    }

    /// Nothing fancy here.
    pub fn tuple(&mut self, tuple: Vec<Spanned<CST>>) -> Result<SST, Syntax> {
        let mut expressions = vec![];
        for expression in tuple {
            expressions.push(self.walk(expression)?)
        }

        Ok(SST::Tuple(expressions))
    }

    pub fn assign(&mut self, pattern: Spanned<CSTPattern>, expression: Spanned<CST>) -> Result<SST, Syntax> {
        let sst_pattern = self.walk_pattern(pattern, false);
        let sst_expression = self.walk(expression)?;

        return Ok(SST::assign(
            sst_pattern,
            sst_expression,
        ));
    }

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
