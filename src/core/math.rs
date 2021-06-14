use crate::common::data::Data;
use crate::core::extract::binop;

/// Adds two numbers, concatenates two strings.
pub fn add(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(l),    Data::Real(r))    => Data::Real(l + r),
        (Data::Integer(l), Data::Integer(r)) => Data::Integer(l + r),
        (Data::String(l),  Data::String(r))  => Data::String(format!("{}{}", l, r)),
        _ => Err("Addition between unsupported datatypes")?,
    };

    return Ok(result);
}

/// Subtraction between two numbers.
pub fn sub(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(l),    Data::Real(r))    => Data::Real(l - r),
        (Data::Integer(l), Data::Integer(r)) => Data::Integer(l - r),
        _ => Err("Subtraction between unsupported datatypes")?,
    };

    return Ok(result);
}

/// Negation of a numbers.
pub fn neg(data: Data) -> Result<Data, String> {
    let result = match data {
        Data::Real(n)    => Data::Real(-n),
        Data::Integer(n) => Data::Integer(-n),
        _ => Err("Subtraction between unsupported datatypes")?,
    };

    return Ok(result);
}

/// Multiplication between two numbers.
pub fn mul(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(l),    Data::Real(r))    => Data::Real(l * r),
        (Data::Integer(l), Data::Integer(r)) => Data::Integer(l * r),
        _ => Err("Multiplication between unsupported datatypes")?,
    };

    return Ok(result);
}

/// Division between two numbers.
/// Raises a runtime error if there is a division by zero.
pub fn div(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(_), Data::Real(n)) if n == 0.0 => Err("Division by zero")?,
        (Data::Real(l), Data::Real(r)) => Data::Real(l / r),
        (Data::Integer(_), Data::Integer(n)) if n == 0 => Err("Division by zero")?,
        (Data::Integer(l), Data::Integer(r)) => Data::Integer(l / r),
        _ => Err("Division between unsupported datatypes")?,
    };

    return Ok(result);
}

/// rem of left operand by right operand division.
/// Raises a runtime error if there is a division by zero.
pub fn rem(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(_),   Data::Real(r)) if r == 0.0 => Err("Division by zero")?,
        (Data::Real(l),   Data::Real(r)) => Data::Real(l.rem_euclid(r)),
        (Data::Integer(_), Data::Integer(n)) if n == 0 => Err("Division by zero")?,
        (Data::Integer(l), Data::Integer(r)) => Data::Integer(l.rem_euclid(r)),
        _ => Err("Division between unsupported datatypes")?,
    };

    return Ok(result);
}

/// Number to a power
pub fn pow(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(l),    Data::Real(r))    => Data::Real(l.powf(r)),
        (Data::Integer(l), Data::Integer(r)) => Data::Integer(l.pow(r as u32)),
        _ => Err("Exponentiation between unsupported datatypes")?,
    };

    return Ok(result);
}