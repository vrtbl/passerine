/// This enum represents a single opcode.
/// Under the hood, it's just a byte.
/// This allows non opcode bytes to be inserted in bytecode streams.
#[repr(u8)]
#[derive(Debug)]
pub enum Opcode {
    /// Load a constant
    Con = 0,
    /// Delete a value off the stack
    Del = 1,
    /// Save a constant into a variable (NOTE: deprecate?)
    Save = 2,
    /// Push a copy of a variable onto the stack
    Load = 3,
    /// Call a function
    Call = 4,
    /// Return from a function
    Return = 5,
}

impl Opcode {
    /// Convert a raw byte to an opcode.
    /// Note that non-opcode bytes should never be interpreted as an opcode.
    /// Under the hood, this is just a transmute, so the regular cautions apply.
    /// This *should* never cause a crash
    /// and if it does, the vm's designed to crash hard
    /// so it'll be pretty obvious.
    pub fn from_byte(byte: u8) -> Opcode {
        unsafe { std::mem::transmute(byte) }
    }
}
