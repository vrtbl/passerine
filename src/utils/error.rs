// TODO: move to a better spot?

use crate::vm::data::Data;

pub struct Error {
    source: &'static str,
    offset: usize,
    data:   Data,
}

impl Error {
    fn display(&self) {
        unimplemented!();
    }
}
