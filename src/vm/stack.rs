use std::collections::HashMap;
use crate::vm::data::Data;
use crate::pipeline::bytecode::Bytecode;

pub type Frame = HashMap<Data, Data>;

// must be under usize in size, so box everything?
// TODO: remove redundant boxes
pub enum Stack {
    Frame(Box<Frame>),
    Lambda(Box<Bytecode>),
    Data(Box<Data>),
}
