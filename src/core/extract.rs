use crate::common::data::Data;

// TODO: macro for data extraction?
// TODO: generalize binop/triop?

pub fn binop(data: Data) -> (Data, Data) {
    match data {
        Data::Tuple(t) if t.len() == 2 => (t[0].clone(), t[1].clone()),
        _ => unreachable!("bad data layout passed to ffi"),
    }
}

pub fn triop(data: Data) -> (Data, Data, Data) {
    match data {
        Data::Tuple(t) if t.len() == 3 => (t[0].clone(), t[1].clone(), t[2].clone()),
        _ => unreachable!("bad data layout passed to ffi"),
    }
}
