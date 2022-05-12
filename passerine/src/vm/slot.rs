use std::{
    cell::RefCell,
    fmt::{
        Debug,
        Formatter,
        Result,
    },
    rc::Rc,
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
    // The topmost frame is stored in the VM.
    Frame,
    // All other frames are suspended.
    Suspend(Suspend),

    // Data
    Data(Data),

    // Uninitialized Data
    NotInit,

    // Refers to a capture stored in the current closure
    Ref(Rc<RefCell<Data>>),
}

impl Slot {
    pub fn data(self) -> Data {
        match self {
            Slot::Data(d) => d,
            Slot::Ref(r) => r.borrow().to_owned(),
            Slot::Frame | Slot::Suspend(_) | Slot::NotInit => {
                unreachable!("expected data on top of stack, found {:?}", self)
            },
        }
    }

    pub fn reference(self) -> Rc<RefCell<Data>> {
        match self {
            Slot::Data(d) => Rc::new(RefCell::new(d)),
            Slot::Ref(r) => r,
            Slot::Frame | Slot::Suspend(_) | Slot::NotInit => {
                unreachable!(
                    "expected reference on top of stack, found {:?}",
                    self
                )
            },
        }
    }
}

impl Debug for Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Slot::Frame => write!(f, "Frame"),
            Slot::Suspend(s) => write!(f, "Suspend({})", s.ip),
            Slot::Data(d) => write!(f, "Data({:?})", d),
            Slot::Ref(r) => write!(f, "Ref({:?})", r),
            Slot::NotInit => write!(f, "NotInit"),
        }
    }
}
