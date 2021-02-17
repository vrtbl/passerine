use std::{
    mem,
    collections::HashMap,
};

use crate::common::span::{Span, Spanned};

use crate::compiler::{
    cst::{CST, CSTPattern},
    sst::{SST, SSTPattern},
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

pub struct Scope {
    locals:   Vec<usize>,
    captures: Vec<usize>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            locals:   vec![],
            captures: vec![],
        }
    }
}

pub struct Hoister {
    /// The unique local symbols in the current scope.
    scopes: Vec<Scope>,
    /// Maps integers (index in vector) to string representation of symbol.
    symbol_table: Vec<String>,
    /// keeps track of variables that were referenced before assignment.
    unresolved_hoists: HashMap<String, usize>,
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
        return self.scopes.pop()
    }

    fn local_scope(&mut self) -> &mut Scope {
        &mut self.scopes[self.scopes.len() - 1]
    }

    fn new_symbol(&mut self, name: String) -> usize {
        let index = self.symbol_table.len();
        self.symbol_table.push(name);
        return index;
    }

    fn resolve_local(&self, name: String) -> Option<usize> {
        for local in self.local_scope().locals.iter() {
            let local_name = self.symbol_table[*local];
            if local_name == name { return Some(*local); }
        }
        return None;
    }

    fn resolve_nonlocal(&mut self, name: String) -> Option<usize> {
        let top_scope = self.exit_scope()?;

        let unique_symbol = if let Some(local) = self.resolve_local(name) {
            local
        } else if let Some(nonlocal) = self.resolve_nonlocal(name) {
            nonlocal
        } else {
            self.reenter_scope(top_scope);
            return None;
        };

        self.reenter_scope(top_scope);
        self.local_scope().captures.push(unique_symbol);
        return Some(unique_symbol);
    }

    /// Returns the unique usize of a local symbol.
    fn resolve(&mut self, name: String, hoist: bool) -> usize {
        // if the symbol has already been defined, use it
        if let Some(local) = self.resolve_local(name) {
            return local;
        } else if let Some(nonlocal) = self.resolve_nonlocal(name) {
            return nonlocal;
        }

        let unique_symbol = match self.unresolved_hoists.get(&name) {
            Some(us) => *us,
            None => self.new_symbol(name),
        };

        if !hoist {
            // declare the local in the current scope
            self.unresolved_hoists.remove(&name);
            self.local_scope().locals.push(unique_symbol);
        } else {
            // mark the variable as used before lexical definition
            self.unresolved_hoists.insert(name, unique_symbol);
        }

        return todo!()
    }

    pub fn walk(&mut self, cst: Spanned<CST>) -> Result<Spanned<SST>, Syntax> {
        let sst: SST = match cst.item {
            CST::Data(data) => SST::Data(data),
            CST::Symbol(name) => self.symbol(name),
            CST::Block(block) => self.block(block)?,
            CST::Label(name, expression) => SST::Label(name, Box::new(self.walk(*expression)?)),
            CST::Tuple(tuple) => self.tuple(tuple),
            CST::FFI    { name,    expression } => SST::ffi(&name, self.walk(*expression)?),
            CST::Assign { pattern, expression } => self.assign(*pattern, *expression),
            CST::Lambda { pattern, expression } => self.lambda(*pattern, *expression),
            CST::Call   { fun,     arg        } => self.call(*fun, *arg),
        };

        return Ok(Spanned::new(sst, cst.span))
    }

    pub fn walk_pattern(&mut self, pattern: Spanned<CSTPattern>) -> Spanned<SSTPattern> {
        let item = match pattern.item {
            CSTPattern::Symbol(name) => SSTPattern::Symbol(self.resolve(name, false)),
            CSTPattern::Data(d)      => SSTPattern::Data(d),
            CSTPattern::Label(n, p)  => SSTPattern::Label(n, Box::new(self.walk_pattern(*p))),
            CSTPattern::Tuple(t)     => SSTPattern::Tuple(
                t.into_iter().map(|c| self.walk_pattern(c)).collect::<Vec<_>>()
            )
        };

        return Spanned::new(item, pattern.span);
    }

    // TODO: merge with resolve?

    /// Replaces a symbol name with a unique identifier for that symbol
    pub fn symbol(&mut self, name: String) -> SST {
        let unique_symbol = self.resolve(name, true);
        return SST::Symbol(unique_symbol);
    }

    pub fn block(&mut self, block: Vec<Spanned<CST>>) -> Result<SST, Syntax> {
        let mut expressions = vec![];
        for expression in block {
            expressions.push(self.walk(expression)?)
        }

        Ok(SST::Block(expressions))
    }

    pub fn lambda(&mut self, pattern: Spanned<CSTPattern>, expression: Spanned<CST>) -> Result<SST, Syntax> {
        self.enter_scope();
        let sst_pattern = self.walk_pattern(pattern);
        let sst_expression = self.walk(expression)?;
        let scope = self.exit_scope().unwrap();

        let lambda = SST::lambda(
            sst_pattern,
            sst_expression,
            scope.locals,
            scope.captures,
        );

        return Ok(lambda);
    }
}
