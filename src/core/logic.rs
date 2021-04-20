use crate::common::data::Data;
use crate::core::extract::binop;

// TODO: implement equality rather than just deriving PartialEq on Data.

// Rust hit it right on the nose with the difference between equality and partial equality
// TODO: equality vs partial equality in passerine?

/// Returns `true` if the `Data` are equal, false otherwise.
pub fn equal(data: Data) -> Result<Data, String> {
    let (left, right) = binop(data);
    return Ok(Data::Boolean(left == right));
}

pub fn greater(data: Data) -> Result<Data, String> {
    // TODO: type coercion
    let result = match binop(data) {
        (Data::Real(left),    Data::Real(right))    => left > right,
        (Data::Integer(left), Data::Integer(right)) => left > right,
        _ => Err("Expected two numbers of the same type")?,
    };

    return Ok(Data::Boolean(result));
}

pub fn less(data: Data) -> Result<Data, String> {
    // TODO: type coercion
    let result = match binop(data) {
        (Data::Real(left),    Data::Real(right))    => left < right,
        (Data::Integer(left), Data::Integer(right)) => left < right,
        _ => Err("Expected two numbers of the same type")?,
    };

    return Ok(Data::Boolean(result));
}

pub fn greater_equal(data: Data) -> Result<Data, String> {
    // TODO: type coercion
    let result = match binop(data) {
        (Data::Real(left),    Data::Real(right))    => left >= right,
        (Data::Integer(left), Data::Integer(right)) => left >= right,
        _ => Err("Expected two numbers of the same type")?,
    };

    return Ok(Data::Boolean(result));
}

pub fn less_equal(data: Data) -> Result<Data, String> {
    // TODO: type coercion
    let result = match binop(data) {
        (Data::Real(left),    Data::Real(right))    => left <= right,
        (Data::Integer(left), Data::Integer(right)) => left <= right,
        _ => Err("Expected two numbers of the same type")?,
    };

    return Ok(Data::Boolean(result));
}
