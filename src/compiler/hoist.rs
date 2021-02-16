use crate::common::span::{Span, Spanned};

use crate::compiler::{
    cst::{Pattern, CST},
    sst::SST,
    // TODO: pattern for where?
    syntax::Syntax,
};

use crate::core::{
    ffi_core,
    ffi::FFI,
};

/// Simple function that a scoped syntax tree (`SST`) from an `CST`.
pub fn hoist(cst: Spanned<CST>) -> Result<Spanned<SST>, Syntax> {
    let ffi = ffi_core();
    let mut hoister = Hoister::new(ffi);
    let sst = hoister.walk(cst)?;
    return Ok(sst);
}



pub struct Hoister {
    /// The locals in the current scope.
    locals: Vec<Local>,
    /// The indicies of captured locals in the current scope
    captures: Vec<usize>,
    /// The nested depth of the current compiler.
    depth: usize,
    /// The foreign functional interface used to bind values
    ffi: FFI,
    /// The FFI functions that have been bound in this scope.
    ffi_names: Vec<String>,
    /// SymbolTable
    symbol_table: Vec<String>,
}

impl Hoister {
    pub fn new(ffi: FFI) -> Hoister {
        Hoister {
            locals:       vec![],
            captures:     vec![],
            depth:        0,
            ffi:          ffi,
            ffi_names:    vec![],
            symbol_table: vec![];
        }
    }

    pub fn walk(&mut self, cst: Spanned<CST>) -> Result<Spanned<SST>, Syntax> {
        match cst.item {
            CST::Data(data) => Ok(SST::Data(data)),
            CST::Symbol(name) => self.symbol(&name, sst.span.clone()),
            CST::Block(block) => self.block(block),
            CST::Print(expression) => self.print(*expression),
            CST::Label(name, expression) => self.label(name, *expression),
            CST::Tuple(tuple) => self.tuple(tuple),
            CST::FFI    { name,    expression } => self.ffi(name, *expression, sst.span.clone()),
            CST::Assign { pattern, expression } => self.assign(*pattern, *expression),
            CST::Lambda { pattern, expression } => self.lambda(*pattern, *expression),
            CST::Call   { fun,     arg        } => self.call(*fun, *arg),
        }
    }

    pub fn symbol(&mut self, name: &str, span: Span) -> {
        
    }
}
