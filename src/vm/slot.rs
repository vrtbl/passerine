use std::fmt::{
    Debug,
    Formatter,
    Result,
};

use crate::common::{
    closure::Closure,
    data::Data,
};

/// Represents a suspended closure.
#[derive(Debug, Clone)]
pub struct Suspend {
    pub ip:      usize,
    pub closure: Closure,
}

/// Represents the value a slot on the VM can take.
#[derive(Clone)]
pub enum Slot {
    // VM Stack
    Frame,
    Suspend(Suspend),

    // Data
    Data(Data),
}

impl Slot {
    pub fn data(self) -> Data {
        match self {
            Slot::Frame => unreachable!("expected data on top of stack, found frame"),
            Slot::Suspend(_) => unreachable!("found suspended frame on top of stack"),
            Slot::Data(d) => d,
        }
    }
}

impl Debug for Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Slot::Frame      => write!(f, "Frame"),
            Slot::Suspend(s) => write!(f, "Suspend({})", s.ip),
            Slot::Data(d)    => write!(f, "Data({:?})", d),
        }
    }
}
