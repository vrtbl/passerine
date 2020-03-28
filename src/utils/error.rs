// TODO: move to a better spot?

use crate::vm::data::Data;

pub enum PResult {
    Ok,
    Error {
        source: &'static str,
        offset: usize,
        data:   Data,
    }
}

impl Display for Error {
    fn display(&self) -> String {

    }
}
