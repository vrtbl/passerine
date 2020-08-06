use std::rc::Rc;
use crate::common::{
    lambda::Lambda,
    data::Data,
};

#[derive(Debug)]
pub struct Closure {
    pub lambda: Lambda,
    pub captured: Vec<Rc<Data>>,
}

impl Closure {
    pub fn wrap(lambda: Lambda) -> Closure {
        Closure {
            lambda,
            captured: 
        }
    }
}
