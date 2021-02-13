// fn data_triop(data: Data) -> (Data, Data) {
//     match data {
//         Data::Tuple(t) if t.len() == 3 => (t[0].clone(), t[1].clone(), t[2].clone()),
//         _ => unreachable!("bad data layout passed to ffi"),
//     }
// }
//
// pub fn ffi_if(data: Data) -> Result<Data, String> {
//     match data_triop(data) {
//         (Data::Boolean(b), a @ Data::Closure(a), Data::Closure(b))
//     }
// }
