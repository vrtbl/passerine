use std::collections::HashMap;
use crate::vm::data::Data;

pub type Stack = Vec<Item>;

// must be under usize in size, so box everything?
// See: https://docs.rs/tagged-box/0.1.1/tagged_box/index.html
// I would like p to be a no-dependancy implementation, however
// TODO: remove redundant boxes
// No clone!
// if data needs to be cloned, clone the data and put it into a new Item
#[derive(Debug, Eq, PartialEq)]
pub enum Item {
    Frame(Box<HashMap<String, Data>>),
    // Lambda(Box<Bytecode>),
    Data(Data),
}

impl Item {
    pub fn frame() -> Item {
        Item::Frame(Box::new(HashMap::new()))
    }

    pub fn data(data: Data) -> Item {
        Item::Data(data)
    }
}
