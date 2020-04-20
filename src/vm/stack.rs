use crate::vm::data::{Data, Tagged};
use crate::vm::local::Local;

pub type Stack = Vec<Item>;

// Implemented NaN-Boxing for Dara
#[derive(Debug)]
pub enum Item {
    Frame,
    // Lambda(Box<Bytecode>),
    // TODO: Locals: stored on the stack, or stored in frames?
    // TODO: box locals?
    Local { local: Local, data: Data },
    Data(Tagged),
}
