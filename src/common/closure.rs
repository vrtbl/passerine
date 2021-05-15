use std::{
    rc::Rc,
    cell::RefCell,
};

use crate::common::{
    lambda::Lambda,
    data::Data,
};

/// Wraps a `Lambda` with some scope context.
/// Each closure is unique when constructed,
/// Because it depends on the surrounding environment it was constructed in.
/// It holds a set of references to variables it captures.
#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub lambda: Rc<Lambda>,
    pub captures: Vec<Rc<RefCell<Data>>>,
}

impl Closure {
    /// Constructs a new `Closure` by wrapping a `Lambda`.
    /// This closure has no captured variables when constructed.
    pub fn wrap(lambda: Rc<Lambda>) -> Closure {
        Closure {
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
