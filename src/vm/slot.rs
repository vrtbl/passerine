use crate::common::{
    closure::Closure,
    data::Data,
};

#[derive(Debug, Clone)]
pub struct Suspend {
    pub ip:      usize,
    pub closure: Closure,
}

#[derive(Debug, Clone)]
pub enum Slot {
    // VM Stack
    Frame,
    Suspend(Suspend),
    NotInit,

    // Data
    Data(Data),
}

impl Slot {
    pub fn data(self) -> Data {
        match self {
            Slot::Frame => unreachable!("expected data on top of stack, found frame"),
            Slot::Suspend(_) => unreachable!("found suspended frame on top of stack"),
            Slot::NotInit => unreachable!("found uninitialized data on top of stack"),
            Slot::Data(d) => d,
        }
    }
}
