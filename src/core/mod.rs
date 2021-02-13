/// This module provides the core language library
/// And compiler-magic FFI bindings.

pub mod ffi;
pub mod math;

use ffi::{FFI, FFIFunction};

// Returns the core FFI used by Passerine.
// Implements basic langauge features, like addition.
pub fn ffi_core() -> FFI {
    let mut ffi = FFI::new();

    ffi.add("add", FFIFunction::new(Box::new(math::ffi_add))).unwrap();
    ffi.add("sub", FFIFunction::new(Box::new(math::ffi_sub))).unwrap();
    ffi.add("mul", FFIFunction::new(Box::new(math::ffi_mul))).unwrap();
    ffi.add("div", FFIFunction::new(Box::new(math::ffi_div))).unwrap();

    return ffi;
}
