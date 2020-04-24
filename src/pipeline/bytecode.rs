// TODO: move to VM
use crate::vm::data::Data;
use crate::vm::local::Local;

#[repr(u8)]
pub enum Opcode {
// § indicates location of op
// note: all args are before op, byte-stream args after
//  Opcode // operands (stack top first) § byte-streams -> Does
    Con    = 0, // § byte-stream      -> pushes value from constant table onto stack
    Save   = 1, // Data § byte-stream -> Stores Data in Symbol
    Load   = 2, // § byte-stream      -> replaces symbol on top of stack with its value
    Clear  = 3, // §                  -> clears stack to last frame
    Call   = 4, // Data Fun §         -> calls the function passing Data as arg
    Return = 5, // Data §             -> clears the frame, leaving value on the stack
}

impl Opcode {
    // NOTE: potentially a lot can go wrong
    // this *should* never cause a crash
    // and if it does, the vm's designed to crash hard
    // so it'll be pretty obvious
    pub fn from_byte(byte: u8) -> Opcode {
        let e: Opcode = unsafe { std::mem::transmute(byte) }; // *chuckles in undefined behavior*
        return e; // "I'm in danger"
    }
}
