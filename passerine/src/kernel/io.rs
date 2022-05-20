use passerine_derive::Inject;

use crate::common::data::Data;

#[derive(Inject)]
pub struct Write(Data);

#[derive(Inject)]
pub struct Writeln(Data);

#[derive(Inject)]
pub struct Show(Data);
