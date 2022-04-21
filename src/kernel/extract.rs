use crate::vm::data::Data;

// TODO: macro for data extraction?
// TODO: generalize binop/triop?

/// Destructures a Rasserine tuple of two items into
/// A Rust tuple of two items.
pub fn binop(data: Data) -> (Data, Data) {
    match data {
        Data::Tuple(t) if t.len() == 2 => (t[0].clone(), t[1].clone()),
        _ => unreachable!("bad data layout passed to ffi"),
    }
}

/// Destructures a Rasserine tuple of three items into
/// A Rust tuple of three items.
pub fn triop(data: Data) -> (Data, Data, Data) {
    match data {
        Data::Tuple(t) if t.len() == 3 => {
            (t[0].clone(), t[1].clone(), t[2].clone())
        },
        _ => unreachable!("bad data layout passed to ffi"),
    }
}
