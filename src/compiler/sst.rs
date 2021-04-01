use crate::common::{
    span::Spanned,
    data::Data,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UniqueSymbol(pub usize);

/// A pattern that mirrors the structure of some Data.
#[derive(Debug, Clone, PartialEq)]
pub enum SSTPattern {
    Symbol(UniqueSymbol),
    Data(Data),
    Label(String, Box<Spanned<SSTPattern>>), // todo usize for label
    Tuple(Vec<Spanned<SSTPattern>>),
    // Where {
    //     pattern: Box<ASTPattern>,
    //     expression: Box<AST>,
    // },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub locals:    Vec<UniqueSymbol>,
    pub nonlocals: Vec<UniqueSymbol>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            locals:    vec![],
            nonlocals: vec![],
        }
    }

    pub fn is_local(&self, unique_symbol: UniqueSymbol) -> bool {
        self.locals.contains(&unique_symbol)
    }

    pub fn is_nonlocal(&self, unique_symbol: UniqueSymbol) -> bool {
        self.nonlocals.contains(&unique_symbol)
    }

    pub fn local_index(&self, unique_symbol: UniqueSymbol) -> Option<usize> {
        self.locals.iter().position(|l| l == &unique_symbol)
    }

    pub fn nonlocal_index(&self, unique_symbol: UniqueSymbol) -> Option<usize> {
        self.nonlocals.iter().position(|l| l == &unique_symbol)
    }
}

/// Represents an item in a hoisted `SST`.
/// Each langauge-level construct has it's own `SST` variant.
/// Note that symbols have been substituted.
/// At this point in compilation the scope of each local is known.
#[derive(Debug, Clone, PartialEq)]
pub enum SST {
    Symbol(UniqueSymbol),
    Data(Data),
    Block(Vec<Spanned<SST>>),
    Assign {
        pattern:    Box<Spanned<SSTPattern>>,
        expression: Box<Spanned<SST>>,
    },
    Lambda {
        pattern:    Box<Spanned<SSTPattern>>,
        expression: Box<Spanned<SST>>,
        scope:      Scope,
    },
    Call {
        fun: Box<Spanned<SST>>,
        arg: Box<Spanned<SST>>,
    },
    Label(String, Box<Spanned<SST>>),
    Tuple(Vec<Spanned<SST>>),
    FFI {
        name:       String,
        expression: Box<Spanned<SST>>,
    },
}

// pub struct ScopeContext {
//     interns: Vec<String>,
// }

impl SST {
    /// Shortcut for creating an `SST::Assign` variant.
    pub fn assign(
        pattern:    Spanned<SSTPattern>,
        expression: Spanned<SST>
    ) -> SST {
        SST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating an `SST::Lambda` variant.
    pub fn lambda(
        pattern:    Spanned<SSTPattern>,
        expression: Spanned<SST>,
        scope:      Scope,
    ) -> SST {
        SST::Lambda {
            pattern:    Box::new(pattern),
            expression: Box::new(expression),
            scope,
        }
    }

    /// Shortcut for creating a `SST::Label` variant.
    pub fn label(name: &str, expression: Spanned<SST>) -> SST {
        SST::Label(name.to_string(), Box::new(expression))
    }

    /// Shortcut for creating a `SST::Lambda` variant.
    pub fn call(fun: Spanned<SST>, arg: Spanned<SST>) -> SST {
        SST::Call {
            fun: Box::new(fun),
            arg: Box::new(arg),
        }
    }

    // Shortcut for creating an `SST::FFI` variant.
    pub fn ffi(name: &str, expression: Spanned<SST>) -> SST {
        SST::FFI {
            name: name.to_string(),
            expression: Box::new(expression),
        }
    }
}
