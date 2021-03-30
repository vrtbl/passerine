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

// TODO: hoist labels? how are labels declared? types?

// NOTE: two goals:
// 1. replace all same-reference symbols with a unique integer
// 2. Build up a table of which symbols are accessible in what scopes.

/// Simple function that a scoped syntax tree (`SST`) from an `CST`.
pub fn hoist(cst: Spanned<CST>) -> Result<Spanned<SST>, Syntax> {
    println!("starting");

    let mut hoister = Hoister::new();
    let sst = hoister.walk(cst)?;

    println!("done hoisting");
    println!("{:#?}", sst);

    if !hoister.unresolved_hoists.is_empty() {
        // TODO: Actual errors
        println!("{:#?}",hoister.unresolved_hoists);
        return Err(Syntax::error(
            &format!(
                "Variable {} referenced before assignment",
                hoister.unresolved_hoists.iter().next().unwrap().0
            ),
            &Span::empty(),
        ))
    }

    return Ok(sst);
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

    fn enter_scope(&mut self) {
        println!("entering");
        self.scopes.push(Scope::new());
    }

    fn reenter_scope(&mut self, scope: Scope) {
        println!("reentering");
        self.scopes.push(scope)
    }

    fn exit_scope(&mut self) -> Option<Scope> {
        println!("exiting");
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
        println!("walking SST");
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
        println!("walking Pattern");
        let item = match pattern.item {
            CSTPattern::Symbol(name) => SSTPattern::Symbol(self.resolve_assign(&name, declare)),
            CSTPattern::Data(d)      => SSTPattern::Data(d),
            CSTPattern::Label(n, p)  => SSTPattern::Label(n, Box::new(self.walk_pattern(*p, declare))),
            CSTPattern::Tuple(t)     => SSTPattern::Tuple(
                t.into_iter().map(|c| self.walk_pattern(c, declare)).collect::<Vec<_>>()
            )
        };

        return Spanned::new(item, pattern.span);
    }

    fn new_symbol(&mut self, name: &str) -> UniqueSymbol {
        let index = self.symbol_table.len();
        self.symbol_table.push(name.to_string());
        println!("Created new symbol {}", index);
        return UniqueSymbol(index);
    }

    fn local_symbol(&self, name: &str) -> Option<UniqueSymbol> {
        for local in self.borrow_local_scope().locals.iter() {
            let local_name = &self.symbol_table[local.0];
            if local_name == name {
                return Some(*local);
            }
        }
        return None;
    }


    fn declare(&mut self, name: &str) -> UniqueSymbol {
        println!("Declaring symbol {}", name);
        let resolved_symbol = match self.unresolved_hoists.get(name) {
            Some(unique_symbol) => *unique_symbol,
            None => {
                println!("symbol is new {}", name);
                let new_symbol = self.new_symbol(name);
                self.local_scope().locals.push(new_symbol);
                return new_symbol;
            },
        };

        println!("declaring Removing from unresolved {}", name);
        // declare the local in the local scope
        // and remove it as unresolved.
        self.local_scope().locals.push(resolved_symbol);
        self.unresolved_hoists.remove(name);

        println!("removing from enclosing");
        println!("{:#?}", self.scopes);
        // remove it as nonlocal in this scope and all enclosing scopes
        for scope in self.scopes[1..].iter_mut() {
            let nonlocal_index = scope.nonlocal_index(resolved_symbol).unwrap();
            scope.nonlocals.remove(nonlocal_index);
        }

        return resolved_symbol;
    }

    // TODO: currently we walk nonlocals on every declaration
    // which is a bit innefficient.

    /// Returns the unique usize of a local symbol.
    /// If a variable is referenced before assignment,
    /// this function will define it in all lexical scopes
    /// once this variable is discovered, we remove the definitions
    /// in all scopes below this one.
    fn resolve_assign(&mut self, name: &str, declare: bool) -> UniqueSymbol {
        println!("resolving assignment for {} declaring: {}", name, declare);

        if declare || self.unresolved_hoists.contains_key(name) {
            return self.declare(name);
        }

        if let Some(unique_symbol) = self.local_symbol(name) { return unique_symbol; }
        println!("symbol not local, searching backwards");

        // if the symbol is defined in the enclosing scope,
        // return that symbol and marked it as being captured by the current scope
        if let Some(scope) = self.exit_scope() {
            let unique_symbol = self.resolve_assign(name, declare);
            self.reenter_scope(scope);
            self.local_scope().nonlocals.push(unique_symbol);
            return unique_symbol;
        }

        println!("declare symbol");


        // if the symbol does not exist in an enclosing scope, we declare it:
        return self.declare(name);
    }

    fn resolve_symbol(&mut self, name: &str) -> UniqueSymbol {
        println!("resolving symbol {}", name);

        if let Some(unique_symbol) = self.local_symbol(name) { return unique_symbol; }

        println!("not local {}", name);

        // if the symbol is defined in the enclosing scope,
        // return that symbol and marked it as being captured by the current scope
        if let Some(scope) = self.exit_scope() {
            let unique_symbol = self.resolve_symbol(name);
            self.reenter_scope(scope);
            self.local_scope().nonlocals.push(unique_symbol);
            return unique_symbol;
        }

        println!("new symbol, unresolved {}", name);

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
        // TODO: make sure that this can shadow outer scopes.
        let sst_pattern = self.walk_pattern(pattern, false);
        let sst_expression = self.walk(expression)?;

        return Ok(SST::assign(
            sst_pattern,
            sst_expression,
        ));
    }

    pub fn lambda(&mut self, pattern: Spanned<CSTPattern>, expression: Spanned<CST>) -> Result<SST, Syntax> {
        self.enter_scope();
        // TODO: make sure that the lambda arguments are redeclared
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
