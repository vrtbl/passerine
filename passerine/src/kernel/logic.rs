use passerine_derive::Effect;

use crate::common::Data;

// TODO: implement equality rather than just deriving PartialEq on Data.

// Rust hit it right on the nose with the difference between equality and
// partial equality TODO: equality vs partial equality in passerine?

#[derive(Effect)]
pub struct Equal(Data, Data);

#[derive(Effect)]
pub struct Less(Data, Data);

#[derive(Effect)]
pub struct Greater(Data, Data);

#[derive(Effect)]
pub struct LessEqual(Data, Data);

#[derive(Effect)]
pub struct GreaterEqual(Data, Data);
