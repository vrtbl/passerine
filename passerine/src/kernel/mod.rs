//! This module provides the standard/core language library
//! And compiler-magic FFI bindings.

// pub mod io;
// pub mod control;
// pub mod logic;

use passerine_derive::Effect;

use crate::common::data::Data;

#[derive(Effect)]
pub struct Write(Data);

// #[derive(Effect)]
// pub struct Writeln(Data);

#[derive(Effect)]
pub struct Show(Data);

#[derive(Effect)]
pub struct Choice {
    cond: bool,
    then: Data,
    other: Data,
}
