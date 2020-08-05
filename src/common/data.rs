use std::{
    hash::{Hash, Hasher},
    fmt::{Debug, Error, Formatter},
    ops::Deref,
    mem,
    f64,
    rc::Rc,
};

use crate::common::{
    chunk::Chunk,
    local::Local,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    // VM Stack
    Frame,
    // Local(Local, Box<Data>),
    // Heap(Local, Rc<Data>),

    // Passerine Data (Atomic)
    Real(f64),
    Boolean(bool),
    String(String),
    Lambda(Chunk),
    Label(String, Box<Data>), // TODO: better type

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
