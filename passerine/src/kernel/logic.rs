use passerine_derive::Inject;

use crate::common::Data;

// TODO: implement equality rather than just deriving PartialEq on Data.

// Rust hit it right on the nose with the difference between equality and
// partial equality TODO: equality vs partial equality in passerine?

#[derive(Inject)]
pub struct Equal(Data, Data);

#[derive(Inject)]
pub struct Less(Data, Data);

#[derive(Inject)]
pub struct Greater(Data, Data);

#[derive(Inject)]
pub struct LessEqual(Data, Data);

#[derive(Inject)]
pub struct GreaterEqual(Data, Data);
