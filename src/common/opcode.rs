/// This enum represents a single opcode.
/// Under the hood, it's just a byte.
/// This allows non opcode bytes to be inserted in bytecode streams.
/// | Opcode | operands, top first | byte-streams | Does                                           |
/// | ------ | ------------------- | ------------ | ---------------------------------------------- |
/// | Con    |                     | Const Index  | Pushes value from constant table onto stack    |
/// | Save   | Data                |              | Stores Data in Symbol                          |
/// | Load   |                     | Local Index  | Replaces symbol on top of stack with its value |
/// | Clear  |                     |              | Clears stack to last frame/local               |
/// | Call   | Fun, Data           |              | Calls the function passing Data as arg         |
/// | Return | Data                |              | Clears the frame, leaving value on the stack   |
#[repr(u8)]
#[derive(Debug)]
pub enum Opcode {
    Con    = 0,
    Save   = 1,
    Load   = 2,
    Call   = 3,
    Return = 4,
    Clear  = 5, // probably unneeded
}

impl Opcode {
    /// Convert a raw byte to an opcode.
    /// Note that non-opcode bytes should never be interpreted as an opcode.
    /// Under the hood, this is just a transmute, so the regular cautions apply.
    /// This *should* never cause a crash
    /// and if it does, the vm's designed to crash hard
    /// so it'll be pretty obvious.
    pub fn from_byte(byte: u8) -> Opcode {
        let e: Opcode = unsafe { std::mem::transmute(byte) }; // *chuckles in undefined behavior*
        return e; // "I'm in danger"
    }
}
