/// This enum represents a single opcode.
/// Under the hood, it's just a byte.
/// This allows non opcode bytes to be inserted in bytecode streams.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    /// Load a constant.
    Con = 0,
    /// Delete a value off the stack.
    Del = 1,
    /// Calls out to a Rust function via FFI
    FFICall = 2,
    /// Copies topmost value on the stack.
    Copy = 3,
    /// Moves a variable onto the heap.
    Capture = 4,
    /// Save a constant into a variable.
    Save = 5,
    /// Save a value to a captured variable.
    SaveCap = 6,
    /// Push a copy of a variable onto the stack.
    Load = 7,
    /// Load a copy of a captured variable.
    LoadCap = 8,
    /// Call a function.
    Call = 9,
    /// Return from a function.
    Return = 10,
    /// Creates a closure over the current local environment.
    Closure = 11,
    /// Prints a value.
    Print = 12,
    /// Constructs a label.
    Label = 13,
    // Constructs a tuple.
    Tuple = 14,
    /// Destructures atomic data by asserting it matches exactly.
    UnData = 15,
    /// Destructures a label.
    UnLabel = 16,
    /// Sestructures a tuple.
    UnTuple = 17,
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
