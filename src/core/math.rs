use crate::common::data::Data;

pub fn ffi_add(data: Data) -> Result<Data, String> {
    let (left, right) = match data {
        Data::Tuple(t) if t.len() == 2 => (t[0].clone(), t[1].clone()),
        _ => unreachable!("bad data layout passed to ffi"),
    };

    let result = match (left, right) {
        (Data::Real(l),   Data::Real(r))   => Data::Real(l + r),
        (Data::String(l), Data::String(r)) => Data::String(format!("{}{}", l, r)),
        _ => Err("Addition between unsupported datatypes")?,
    };

    return Ok(result);
}
