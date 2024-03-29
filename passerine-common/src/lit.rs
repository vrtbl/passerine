use std::{
    f64,
    fmt::{Debug, Display, Formatter, Result},
};

use crate::data::Data;

pub enum ArbInt {
    Small(u128),
    Large(Vec<u128>),
}

/// Built-in Passerine datatypes.
#[derive(Debug, Clone, PartialEq, proptest_derive::Arbitrary)]
pub enum Lit {
    // TODO: switch to this:
    // Number Literals
    // Float {
    //     point:    usize,
    //     mantissa: ArbInt,
    // },
    // Integer(ArbInt),
    Float(f64),
    Integer(i64),

    /// A UTF-8 encoded string.
    String(String),

    /// A Label is similar to a type, and wraps some data.
    /// in the future labels will have associated namespaces.
    #[proptest(skip)]
    Label(usize, Box<Lit>),

    // Compound Datatypes
    /// The empty Tuple
    Unit, // an empty typle
    Boolean(bool),
}

impl Lit {
    pub fn to_data(self) -> Data {
        match self {
            Lit::Float(f) => Data::Float(f),
            Lit::Integer(i) => Data::Integer(i),
            Lit::String(s) => Data::String(s),
            Lit::Label(_, _) => todo!(),
            Lit::Unit => Data::Unit,
            Lit::Boolean(b) => Data::Boolean(b),
        }
    }
}

impl Display for Lit {
    /// Displays some Passerine Data in a pretty manner, as if it were printed
    /// to console.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Lit::Float(n) => write!(f, "{}", n),
            Lit::Integer(n) => write!(f, "{}", n),
            Lit::String(s) => write!(f, "{}", s),
            // TODO: better representation for Labels
            Lit::Label(n, v) => write!(f, "#{}({})", n, v),
            Lit::Unit => write!(f, "()"),
            Lit::Boolean(b) => {
                write!(f, "{}", if *b { "True" } else { "False" })
            }
        }
    }
}
