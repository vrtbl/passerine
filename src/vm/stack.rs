use crate::vm::data::{Data, Tagged};
use crate::vm::local::Local;

pub type Stack = Vec<Item>;

// Implemented NaN-Boxing for Data
/// The different items that can be on a stack.
/// Note that this is quite big, might tag for nan-boxing.
#[derive(Debug)]
pub enum Item {
    Frame,
    // TODO: Locals: stored on the stack, or stored in frames?
    // TODO: box locals?
    Local { local: Local, data: Data },
    Data(Tagged),
}
