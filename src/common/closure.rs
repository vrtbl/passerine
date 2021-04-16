use std::{
    rc::Rc,
    cell::RefCell,
};

use crate::common::{
    stamp::stamp,
    lambda::Lambda,
    data::Data,
};

// TODO: take a reference to lambda?

/// Wraps a `Lambda` with some scope context.
/// Each closure is unique when constructed,
/// Because it depends on the surrounding environment it was constructed in.
/// It holds a set of references to variables it captures.
#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub id: String,
    pub lambda: Lambda,
    pub captures: Vec<Rc<RefCell<Data>>>,
}

impl Closure {
    /// Constructs a new `Closure` by wrapping a `Lambda`.
    /// This closure has no captured variables when constructed.
    pub fn wrap(lambda: Lambda) -> Closure {
        Closure {
            id: stamp(0),
            lambda,
            captures: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::common::lambda::Lambda;

    // #[test]
    // fn unique() {
    //     let lambda = Lambda::empty();
    //     let a = Closure::wrap(lambda.clone());
    //     let b = Closure::wrap(lambda.clone());
    //
    //     assert_ne!(a.id, b.id);
    // }
}
