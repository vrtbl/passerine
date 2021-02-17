use std::mem;

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
    lambdas:  Vec<Box<dyn FnOnce() -> Result<(), Syntax>>>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            locals:   vec![],
            captures: vec![],
            lambdas:  vec![],
        }
    }
}

pub struct Hoister {
    /// The unique local symbols in the current scope.
    scopes: Vec<Scope>,
    /// Maps integers (index in vector) to string representation of symbol.
    symbol_table: Vec<String>,
}

impl Hoister {
    pub fn new() -> Hoister {
        Hoister {
            scopes:       vec![Scope::new()],
            // captures:     vec![],
            symbol_table: vec![],
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn exit_scope(&mut self) -> Scope {
        return self.scopes.pop().unwrap();
    }

    fn declare(&mut self, name: String) -> usize {
        let unique_symbol = self.symbol_table.len();
        self.symbol_table.push(name);
        self.scopes[self.scopes.len() - 1].locals.push(unique_symbol);
        return unique_symbol;
    }

    /// Returns the unique usize of a local symbol.
    fn resolve(&self, name: String) -> usize {
        // look backwards for the first matching name
        for scope in self.scopes.iter().rev() {
            for unique_symbol in scope.locals.iter() {
                // if the name is defined in the current scope
                let symbol_name = self.symbol_table[*unique_symbol];
                if symbol_name == name {
                    // replace the symbol with it's integer
                    return *unique_symbol;
                }
            }
        }

        // if not, preemptively declare the symbol
        return self.declare(name.to_string());
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
            CSTPattern::Symbol(name) => SSTPattern::Symbol(self.resolve(name)),
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
        let unique_symbol = self.resolve(name);
        return SST::Symbol(unique_symbol);
    }

    pub fn block(&mut self, block: Vec<Spanned<CST>>) -> Result<SST, Syntax> {
        let mut expressions = vec![];
        for expression in block {
            expressions.push(self.walk(expression)?)
        }

        Ok(SST::Block(expressions))
    }

    pub fn lambda(&mut self, pattern: Spanned<CSTPattern>, expression: Spanned<CST>) -> Result<CST, Syntax> {
        let mut lambda = SST::empty_lambda();

        // have no idea if this will work
        let callback = || -> Result<(), Syntax> {
            self.enter_scope();
            let sst_pattern = self.walk_pattern(pattern);
            let sst_expression = self.walk(expression)?;
            if let SST::Lambda { ref mut pattern, ref mut expression, .. } = lambda {
                *pattern = Box::new(sst_pattern);
                *expression = Box::new(sst_expression);
            } else {
                unreachable!()
            }
            Ok(())
        };

        self.scopes[self.scopes.len() - 1].lambdas.push(Box::new(callback));
        todo!()
    }
}
