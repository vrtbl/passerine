//! This module provides the standard/core language library
//! And compiler-magic FFI bindings.

// pub mod io;
// pub mod control;
// pub mod logic;

use passerine_derive::Inject;

use crate::common::data::Data;

// TODO: rename Inject to Effect

#[derive(Inject)]
pub struct Write(Data);

// #[derive(Inject)]
// pub struct Writeln(Data);

#[derive(Inject)]
pub struct Show(Data);

#[derive(Inject)]
pub struct Choice {
    cond: bool,
    then: Data,
    other: Data,
}
