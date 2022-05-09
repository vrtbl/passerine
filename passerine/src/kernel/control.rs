use passerine_derive::Inject;

use crate::common::data::Data;

#[derive(Inject)]
pub struct Choice {
    cond:  bool,
    then:  Data,
    other: Data,
}
