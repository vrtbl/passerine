use crate::common::data::Data;

#[derive(Debug, Clone)]
pub enum Slot {
    // VM Stack
    Frame,
    NotInit,

    // Data
    Data(Data),
}

impl Slot {
    pub fn data(self) -> Data {
        match self {
            Slot::Frame => unreachable!("expected data on top of stack, found frame"),
            Slot::NotInit => unreachable!("found uninitialized data on top of stack"),
            Slot::Data(d) => d,
        }
    }
}
