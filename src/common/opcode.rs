/// This enum represents a single opcode.
/// Under the hood, it's just a byte.
/// This allows non opcode bytes to be inserted in bytecode streams.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    /// Load a constant.
    Con = 0,
    /// Load uninitialized Data.
    NotInit = 1,
    /// Delete a value off the stack.
    Del = 2,
    /// Calls out to a Rust function via FFI
    FFICall = 3,
    /// Copies topmost value on the stack.
    Copy = 4,
    /// Moves a variable onto the heap.
    Capture = 5,
    /// Save a constant into a variable.
    Save = 6,
    /// Save a value to a captured variable.
    SaveCap = 7,
    /// Push a copy of a variable onto the stack.
    Load = 8,
    /// Load a copy of a captured variable.
    LoadCap = 9,
    /// Call a function.
    Call = 10,
    /// Return from a function.
    Return = 11,
    /// Creates a closure over the current local environment.
    Closure = 12,
    /// Prints a value.
    Print = 13,
    /// Constructs a label.
    Label = 14,
    // Constructs a tuple.
    Tuple = 15,
    /// Destructures atomic data by asserting it matches exactly.
    UnData = 16,
    /// Destructures a label.
    UnLabel = 17,
    /// Sestructures a tuple.
    UnTuple = 18,
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
