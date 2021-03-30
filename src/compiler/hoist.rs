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
    let mut hoister = Hoister::new();
    let sst = hoister.walk(cst)?;

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
        self.scopes.push(Scope::new());
    }

    fn reenter_scope(&mut self, scope: Scope) {
        self.scopes.push(scope)
    }

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
        let resolved_symbol = match self.unresolved_hoists.get(name) {
            Some(unique_symbol) => *unique_symbol,
            None => {
                let new_symbol = self.new_symbol(name);
                self.local_scope().locals.push(new_symbol);
                return new_symbol;
            },
        };

        // declare the local in the local scope
        // and remove it as unresolved.
        self.local_scope().locals.push(resolved_symbol);
        self.unresolved_hoists.remove(name);

        // remove it as nonlocal in this scope and all enclosing scopes
        if let Some(nonlocal_index) = self.local_scope().nonlocal_index(resolved_symbol) {
            self.local_scope().nonlocals.remove(nonlocal_index);
        } else {
            // TODO: should be unreachable
            todo!();
        }

        if let Some(scope) = self.exit_scope() {
            let unique_symbol = self.resolve_symbol(name);
            self.reenter_scope(scope);
            self.local_scope().nonlocals.push(unique_symbol);
            return unique_symbol;
        }

        todo!()
    }

    /// Returns the unique usize of a local symbol.
    /// If a variable is referenced before assignment,
    /// this function will define it in all lexical scopes
    /// once this variable is discovered, we remove the definitions
    /// in all scopes below this one.
    fn resolve_assign(&mut self, name: &str, declare: bool) -> UniqueSymbol {
        if declare || self.unresolved_hoists.contains_key(name) {
            return self.declare(name);
        }

        if let Some(unique_symbol) = self.local_symbol(name) { return unique_symbol; }

        // if the symbol is defined in the enclosing scope,
        // return that symbol and marked it as being captured by the current scope
        if let Some(scope) = self.exit_scope() {
            let unique_symbol = self.resolve_symbol(name);
            self.reenter_scope(scope);
            self.local_scope().nonlocals.push(unique_symbol);
            return unique_symbol;
        }

        match self.unresolved_hoists.get(name) {

        }

        todo!()
    }

    fn resolve_symbol(&mut self, name: &str) -> UniqueSymbol {
        if let Some(unique_symbol) = self.local_symbol(name) { return unique_symbol; }

        // if the symbol is defined in the enclosing scope,
        // return that symbol and marked it as being captured by the current scope
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
