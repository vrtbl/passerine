use std::collections::{BTreeMap, BTreeSet};

mod heap;
pub use heap::{Pointer, Heap};

mod stack;
// mod fiber;
mod code;
mod slot;

pub use slot::Slot;

// pub struct Code {
//     bytes: Vec<u8>,
// }
//
// pub struct Worker {
//     code_pool:     BTreeMap<CodeId, Code>,
//     constant_pool: BTreeMap<ConstantId, Constant>,
//     process_pool:  BTreeMap<FiberId, Fiber>,
// }
//
// macro_rules! op {
//     {
//         fn $name:ident(
//             $ip:ident,
//             $next_op:ident,
//             $stack:ident,
//             $heap:ident,
//             $code:ident,
//         ) $body:expr
//     } => {
//         pub fn name(
//             ip: &mut usize,
//             next_op: &mut OpCode,
//             stack: &mut Stack,
//             heap: &mut Heap,
//             code: &Code,
//         ) -> std::option::Option<HandlerId> {
//             body
//         }
//     };
// }
//
// op! {
//     fn add_u64(
//         ip,
//         next_op,
//         stack,
//         heap,
//         code,
//     ) {
//         *next_op = code.prefetch(ip);
//         let a = stack.pop();
//         let b = stack.pop();
//         stack.push(a + b);
//         None
//     }
//
// }
//
// pub fn add_u64(
//     ip: &mut usize,
//     next_op: &mut OpCode,
//     stack: &mut Stack,
//     heap: &mut Heap,
//     code: &Code,
// ) -> Option<HandlerId> {
//     *next_op = code.prefetch(ip);
//     let a = stack.pop();
//     let b = stack.pop();
//     stack.push(a + b);
//     None
// }

pub fn main() {
    todo!();
}
