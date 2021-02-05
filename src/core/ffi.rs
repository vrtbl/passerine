use std::{
    rc::Rc,
    collections::HashMap,
};
use crate::common::data::Data;

pub struct FFIFunction(Rc<dyn Fn(Data) -> Result<Data, String>>);

impl FFIFunction {
    pub fn new(function: Box<dyn Fn(Data) -> Result<Data, String>>) -> FFIFunction {
        FFIFunction(Rc::new(function))
    }

    #[inline]
    pub fn call(&self, data: Data) -> Result<Data, String> {
        (self.0)(data)
    }
}

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

/// A foreign functional interface, mapping names to functions,
/// passed to the compiler at the bytecode generation step.
pub struct FFI(HashMap<String, FFIFunction>);

impl FFI {
    /// Creates a new empty Foreign Functional Interface.
    pub fn new() -> FFI {
        FFI(HashMap::new())
    }

    /// Returns true if the function has already been added to the `FFI`.
    pub fn add(&mut self, name: &str, function: FFIFunction) -> bool {
        match self.0.insert(name.to_string(), function) {
            Some(x) => false,
            None => true,
        }
    }
}
