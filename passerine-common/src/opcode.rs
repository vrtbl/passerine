/// This enum represents a single opcode.
/// Under the hood, it's just a byte.
/// This allows non opcode bytes to be inserted in bytecode streams.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    /// Load a constant.
    Con     = 0,
    /// Load uninitialized Data.
    NotInit = 1,
    /// Delete a value off the stack.
    Del     = 2,
    /// Calls out to a Rust function via FFI
    FFICall = 3,
    /// Copies topmost value on the stack.
    Copy    = 4,
    /// Moves a variable onto the heap.
    Capture = 5,
    /// Save a constant into a variable.
    Save    = 6,
    /// Save a value to a captured variable.
    SaveCap = 7,
    /// Push a copy of a variable onto the stack.
    Load    = 8,
    /// Load a copy of a captured variable.
    LoadCap = 9,
    /// Call a function.
    Call    = 10,
    /// Return from a function.
    Return  = 11,
    /// Creates a closure over the current local environment.
    Closure = 12,
    /// Prints a value.
    Print   = 13,
    ///
    Handler = 14,
    ///
    Effect  = 15,
    /// Constructs a label.
    Label   = 16,
    /// Constructs a tuple.
    Tuple   = 17,
    ///
    Record  = 18,
    /// Destructures atomic data by asserting it matches exactly.
    UnData  = 19,
    // TODO: make unlabel take the label index as an arg.
    /// Destructures a label.
    UnLabel = 20,
    /// Destructures a tuple.
    UnTuple = 21,
    /// Add two numbers on the stack.
    Add     = 22,
    /// Subtract two numbers on the stack.
    Sub     = 23,
    /// Negate a number.
    Neg     = 24,
    /// Multiple two numbers on the stack.
    Mul     = 25,
    /// Divide two numbers, raising ZeroDiv side effect
    Div     = 26,
    /// Take the remainder of two numbers, raising ZeroDiv side effect
    Rem     = 27,
    /// Take a number to a power.
    Pow     = 28,
    /// Does nothing. Must always be last.
    Noop    = 29,
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

    /// Convert a raw byte to an opcode.
    /// Performing a bounds check first.
    /// Used for bytecode verification,
    /// Do not use in a hot loop.
    pub fn from_byte_safe(byte: u8) -> Option<Opcode> {
        // safety: we do a bounds check on the byte
        if byte <= Opcode::Noop as u8 {
            Some(Opcode::from_byte(byte))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe() {
        assert_eq!(None, Opcode::from_byte_safe((Opcode::Noop as u8) + 1));
        assert_eq!(
            Some(Opcode::Noop),
            Opcode::from_byte_safe(Opcode::Noop as u8)
        );
    }
}
