//! This module provides the standard/core language library
//! And compiler-magic FFI bindings.

pub mod ffi;
pub mod extract;
pub mod effect;
pub mod inject;

pub mod math;
pub mod io;
pub mod control;
pub mod logic;

use ffi::{
    FFIFunction,
    FFI,
};

// Returns the core FFI used by Passerine.
// Implements basic langauge features, like addition.
pub fn ffi_core() -> FFI {
    let mut ffi = FFI::new();

    // math
    ffi.add("add", FFIFunction::new(Box::new(math::add)))
        .unwrap();
    ffi.add("sub", FFIFunction::new(Box::new(math::sub)))
        .unwrap();
    ffi.add("neg", FFIFunction::new(Box::new(math::neg)))
        .unwrap();
    ffi.add("mul", FFIFunction::new(Box::new(math::mul)))
        .unwrap();
    ffi.add("div", FFIFunction::new(Box::new(math::div)))
        .unwrap();
    ffi.add("rem", FFIFunction::new(Box::new(math::rem)))
        .unwrap();
    ffi.add("pow", FFIFunction::new(Box::new(math::pow)))
        .unwrap();

    // io
    ffi.add("println", FFIFunction::new(Box::new(io::println)))
        .unwrap();
    ffi.add("print", FFIFunction::new(Box::new(io::print)))
        .unwrap();
    ffi.add("to_string", FFIFunction::new(Box::new(io::to_string)))
        .unwrap();

    // control
    ffi.add("if", FFIFunction::new(Box::new(control::if_choice)))
        .unwrap();

    // logic
    ffi.add("equal", FFIFunction::new(Box::new(logic::equal)))
        .unwrap();
    ffi.add("greater", FFIFunction::new(Box::new(logic::greater)))
        .unwrap();
    ffi.add("less", FFIFunction::new(Box::new(logic::less)))
        .unwrap();
    ffi.add(
        "greater_equal",
        FFIFunction::new(Box::new(logic::greater_equal)),
    )
    .unwrap();
    ffi.add("less_equal", FFIFunction::new(Box::new(logic::less_equal)))
        .unwrap();

    return ffi;
}
