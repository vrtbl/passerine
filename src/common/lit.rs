use std::{
    fmt::{
        Debug,
        Display,
        Formatter,
        Result,
    },
    f64,
    rc::Rc,
    cell::RefCell,
};

use crate::common::{
    lambda::Lambda,
    closure::Closure,
};

pub struct ArbInt {
    Small(u128),
    Large(Vec<u128>),
}

// TODO: separate VM data from parser data

/// Built-in Passerine datatypes.
#[derive(Clone, PartialEq)]
pub enum Lit {
    // Number Literals
    Float {
        point:    usize,
        mantissa: ArbInt,
    },
    Integer(ArbInt),

    /// A UTF-8 encoded string.
    String(String),

    /// A Label is similar to a type, and wraps some data.
    /// in the future labels will have associated namespaces.
    Label(usize, Box<Lit>),

    // Compound Datatypes
    /// The empty Tuple
    Unit, // an empty typle
}

// TODO: manually implement the equality trait
// NOTE: might have to implement partial equality as well
// NOTE: equality represents passerine equality, not rust equality
impl Eq for Data {}

impl Display for Data {
    /// Displays some Passerine Data in a pretty manner, as if it were printed to console.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Data::Heaped(_)   => unreachable!("Can not display heaped data"),
            Data::NotInit     => unreachable!("Can not display uninitialized data"),
            Data::Float(n)     => write!(f, "{}", n),
            Data::Integer(n)  => write!(f, "{}", n),
            Data::Boolean(b)  => write!(f, "{}", if *b { "true" } else { "false" }),
            Data::String(s)   => write!(f, "{}", s),
            Data::Lambda(_)   => unreachable!("Can not display naked functions"),
            Data::Closure(_)  => write!(f, "Function"),
            Data::Kind(_)     => unreachable!("Can not display naked labels"),
            Data::Label(n, v) => write!(f, "{} {}", n, v),
            Data::Unit        => write!(f, "()"),
            Data::Tuple(t)    => write!(f, "({})", t.iter()
                .map(|i| format!("{}", i))
                .collect::<Vec<String>>()
                .join(", ")
            ),
        }
    }
}



impl Debug for Data {
    /// Displays some Passerine Data following Rust conventions,
    /// with certain fields omitted.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Data::Heaped(h)   => write!(f, "Heaped({:?})", h.borrow()),
            Data::NotInit     => write!(f, "NotInit"),
            Data::Float(n)     => write!(f, "Float({:?})", n),
            Data::Integer(n)  => write!(f, "Integer({:?})", n),
            Data::Boolean(b)  => write!(f, "Boolean({:?})", b),
            Data::String(s)   => write!(f, "String({:?})", s),
            Data::Lambda(_)   => write!(f, "Function(...)"),
            Data::Closure(_c) => write!(f, "Closure(...)"), // TODO: how to differentiate?
            Data::Kind(n)     => write!(f, "Kind({})", n),
            Data::Label(n, v) => write!(f, "Label({}, {:?})", n, v),
            Data::Unit        => write!(f, "Unit"),
            Data::Tuple(t)    => write!(f, "Tuple({:?})", t),
        }
    }
}
