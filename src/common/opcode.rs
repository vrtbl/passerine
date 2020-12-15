/// This enum represents a single opcode.
/// Under the hood, it's just a byte.
/// This allows non opcode bytes to be inserted in bytecode streams.
#[repr(u8)]
#[derive(Debug)]
pub enum Opcode {
    /// Load a constant.
    Con = 0,
    /// Delete a value off the stack.
    Del = 1,
    /// Copies topmost value on the stack.
    Copy = 2,
    /// Moves a variable onto the heap.
    Capture = 3,
    /// Save a constant into a variable.
    Save = 4,
    /// Save a value to a captured variable.
    SaveCap = 5,
    /// Push a copy of a variable onto the stack.
    Load = 6,
    /// Load a copy of a captured variable.
    LoadCap = 7,
    /// Call a function.
    Call = 8,
    /// Return from a function.
    Return = 9,
    /// Creates a closure over the current local environment.
    Closure = 10,
    /// Prints a value.
    Print = 11,
    /// Constructs a label.
    Label = 12,
    /// Destructures a label.
    UnLabel = 13,
    /// Destructures atomic data by asserting it matches exactly
    UnData = 14,
    /// Calls out to a Rust function via FFI
    FFICall = 15,
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
