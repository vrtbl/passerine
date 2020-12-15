use std::rc::Rc;
use crate::common::data::Data;

pub struct FFIFunction(pub Rc<dyn Fn(Data) -> Result<Data, String>>);

impl std::fmt::Debug for FFIFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FFIFunction(...)")
    }
}

impl PartialEq for FFIFunction {
    fn eq(&self, _other: &FFIFunction) -> bool {
        return false;
    }
}
