use crate::common::{
    span::Spanned,
    data::Data,
};

// TODO: SST Pattern

/// Represents an item in a hoisted `SST`.
/// Each langauge-level construct has it's own `SST` variant.
/// Note that symbols have been substituted.
/// At this point in compilation the scope of each local is known.
#[derive(Debug, Clone, PartialEq)]
pub enum SST {
    Symbol(usize),
    Data(Data),
    Block(Vec<Spanned<SST>>),
    Assign {
        pattern:    Box<Spanned<CSTPattern>>,
        expression: Box<Spanned<SST>>,
    },
    Lambda {
        pattern:    Box<Spanned<CSTPattern>>,
        expression: Box<Spanned<SST>>,
        // TODO: just locals, or all variables accessible?
        locals:     Vec<usize>, // unique usizes of locals defined in this scope
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

pub struct ScopeContext {
    interns: Vec<String>,
}

impl SST {
    /// Shortcut for creating an `SST::Assign` variant.
    pub fn assign(
        pattern:    Spanned<CSTPattern>,
        expression: Spanned<SST>
    ) -> SST {
        SST::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    /// Shortcut for creating an `SST::Lambda` variant.
    pub fn lambda(
        pattern:    Spanned<CSTPattern>,
        expression: Spanned<SST>
    ) -> SST {
        SST::Lambda {
            pattern:    Box::new(pattern),
            expression: Box::new(expression),
            locals:     vec![],
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
