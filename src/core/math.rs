use crate::common::data::Data;
use crate::core::extract::binop;

pub fn add(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(l),   Data::Real(r))   => Data::Real(l + r),
        (Data::String(l), Data::String(r)) => Data::String(format!("{}{}", l, r)),
        _ => Err("Addition between unsupported datatypes")?,
    };

    return Ok(result);
}

pub fn sub(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(l),   Data::Real(r))   => Data::Real(l - r),
        _ => Err("Subtraction between unsupported datatypes")?,
    };

    return Ok(result);
}

pub fn mul(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(l),   Data::Real(r))   => Data::Real(l * r),
        _ => Err("Multiplication between unsupported datatypes")?,
    };

    return Ok(result);
}

pub fn div(data: Data) -> Result<Data, String> {
    let result = match binop(data) {
        (Data::Real(_),   Data::Real(n)) if n == 0.0 => Err("Division by zero")?,
        (Data::Real(l),   Data::Real(r)) => Data::Real(l / r),
        _ => Err("Division between unsupported datatypes")?,
    };

    return Ok(result);
}
