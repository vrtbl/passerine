use std::mem;

use crate::common::span::{Span, Spanned};

use crate::compiler::{
    cst::CST,
    sst::SST,
    pattern::CSTPattern,
    // TODO: pattern for where?
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
    /// The unique symbols in the current scope.
    scopes: Vec<Vec<usize>>,
    /// The indicies of captured locals in the current scope
    // captures: Vec<usize>,
    /// The nested depth of the current compiler.
    depth: usize,
    /// Maps integers (index in vector) to string representation of symbol.
    symbol_table: Vec<String>,
}

impl Hoister {
    pub fn new() -> Hoister {
        Hoister {
            scopes:       vec![vec![]],
            // captures:     vec![],
            depth:        0,
            symbol_table: vec![],
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(vec![]);
        self.depth += 1;
    }

    fn exit_scope(&mut self) -> Vec<usize> {
        self.depth -= 1;
        return self.scopes.pop().unwrap();
    }

    fn declare(&mut self, name: String) -> usize {
        let unique_symbol = self.symbol_table.len();
        self.symbol_table.push(name);
        self.scopes[self.scopes.len() - 1].push(unique_symbol);
        return unique_symbol;
    }

    /// Returns the unique usize of a local symbol.
    fn resolve(&self, name: String) -> usize {
        // look backwards for the first matching name
        for scope in self.scopes.iter().rev() {
            for unique_symbol in scope {
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
        }

        return Ok(Spanned::new(sst, cst.span))
    }

    // TODO: merge with resolve?

    /// Replaces a symbol name with a unique identifier for that symbol
    pub fn symbol(&mut self, name: String) ->SST {
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

    pub fn assign(&mut self, pattern: CSTPattern, expression:CST) -> Result<CST, Syntax> {

    }
}
