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

/// Built-in Passerine datatypes.
#[derive(Clone, PartialEq)]
pub enum Data {
    // VM Stack
    Frame,
    NotInit,
    Heaped(Rc<RefCell<Data>>),

    // Passerine Data (Atomic)
    Real(f64),
    Boolean(bool),
    String(String),
    // TODO: make lambda Rc?
    Lambda(Box<Lambda>),
    Closure(Box<Closure>),

    // TODO: rework how labels and tags work
    // Kind is the base component of an unconstructed label
    Kind(String),
    Label(Box<String>, Box<Data>),

    // Compound Datatypes
    Unit, // an empty typle
    // Tuple(Vec<Data>),
    // // TODO: Hashmap?
    // // I mean, it's overkill for small things
    // // yet if people have very big records, yk.
    // Record(Vec<(Local, Data)>),
    // ArbInt(ArbInt),
}

// TODO: manually implement the equality trait
// NOTE: might have to implement partial equality as well
// NOTE: equality represents passerine equality, not rust equality
impl Eq for Data {}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Data::Frame       => unreachable!("Can not display stack frame"),
            Data::NotInit     => unreachable!("Can not display unitialized data"),
            Data::Heaped(_)   => unreachable!("Can not display heaped data"),
            Data::Real(n)     => write!(f, "{}", n),
            Data::Boolean(b)  => write!(f, "{}", if *b { "true" } else { "false" }),
            Data::String(s)   => write!(f, "{}", s),
            Data::Lambda(_)   => unreachable!("Can not display naked functions"),
            Data::Closure(c)  => write!(f, "Function ~ {}", c.id),
            Data::Kind(_)     => unreachable!("Can not display naked labels"),
            Data::Label(n, v) => write!(f, "{} {}", n, v),
            Data::Unit        => write!(f, "()"),
        }
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Data::Frame       => write!(f, "Frame"),
            Data::NotInit     => write!(f, "NotInit"),
            Data::Heaped(h)   => write!(f, "Heaped({:?})", h.borrow()),
            Data::Real(n)     => write!(f, "Real({:?})", n),
            Data::Boolean(b)  => write!(f, "Boolean({:?})", b),
            Data::String(s)   => write!(f, "String({:?})", s),
            Data::Lambda(_)   => write!(f, "Function(...)"),
            Data::Closure(c)  => write!(f, "Closure({})", c.id),
            Data::Kind(n)     => write!(f, "Kind({})", n),
            Data::Label(n, v) => write!(f, "Label({}, {:?})", n, v),
            Data::Unit        => write!(f, "Unit"),
        }
    }
}
