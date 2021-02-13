use crate::common::data::Data;

// TODO: macro for data extraction

fn data_binop(data: Data) -> (Data, Data) {
    match data {
        Data::Tuple(t) if t.len() == 2 => (t[0].clone(), t[1].clone()),
        _ => unreachable!("bad data layout passed to ffi"),
    }
}

pub fn ffi_add(data: Data) -> Result<Data, String> {
    let result = match data_binop(data) {
        (Data::Real(l),   Data::Real(r))   => Data::Real(l + r),
        (Data::String(l), Data::String(r)) => Data::String(format!("{}{}", l, r)),
        _ => Err("Addition between unsupported datatypes")?,
    };

    return Ok(result);
}

pub fn ffi_sub(data: Data) -> Result<Data, String> {
    let result = match data_binop(data) {
        (Data::Real(l),   Data::Real(r))   => Data::Real(l - r),
        _ => Err("Subtraction between unsupported datatypes")?,
    };

    return Ok(result);
}

pub fn ffi_mul(data: Data) -> Result<Data, String> {
    let result = match data_binop(data) {
        (Data::Real(l),   Data::Real(r))   => Data::Real(l * r),
        _ => Err("Multiplication between unsupported datatypes")?,
    };

    return Ok(result);
}

pub fn ffi_div(data: Data) -> Result<Data, String> {
    let result = match data_binop(data) {
        (Data::Real(_),   Data::Real(n)) if n == 0.0 => Err("Division by zero")?,
        (Data::Real(l),   Data::Real(r)) => Data::Real(l / r),
        _ => Err("Division between unsupported datatypes")?,
    };

    return Ok(result);
}
