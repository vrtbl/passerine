use std::{
    rc::Rc,
    cell::RefCell,
};

use crate::common::{
    stamp::stamp,
    lambda::Lambda,
    data::Data,
};

/// Wraps a `Lambda` with some scope context.
/// > NOTE: currently a work-in-progress.
#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub id: String,
    pub lambda: Lambda,
    pub captureds: Vec<Rc<RefCell<Data>>>,
}

impl Closure {
    /// Constructs a new `Closure` by wrapping a `Lambda`.
    pub fn wrap(lambda: Lambda) -> Closure {
        Closure { id: stamp(0), lambda, captureds: vec![] }
    }
}
