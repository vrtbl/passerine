/// This enum represents a single opcode.
/// Under the hood, it's just a byte.
/// This allows non opcode bytes to be inserted in bytecode streams.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    /// Load a constant.
    Con = 0,
    /// Load uninitialized Data.
    NotInit,
    /// Delete a value off the stack.
    Del,
    /// Calls out to a Rust function via FFI
    FFICall,
    /// Copies topmost value on the stack.
    Copy,
    /// Moves a variable onto the heap.
    Capture,
    /// Save a constant into a variable.
    Save,
    /// Save a value to a captured variable.
    SaveCap,
    /// Push a copy of a variable onto the stack.
    Load,
    /// Load a copy of a captured variable.
    LoadCap,
    /// Call a function.
    Call,
    /// Return from a function.
    Return,
    /// Creates a closure over the current local environment.
    Closure,
    /// Prints a value.
    Print,
    ///
    Handler,
    ///
    Effect,
    /// Constructs a label.
    Label,
    /// Constructs a tuple.
    Tuple,
    ///
    Record,
    /// Destructures atomic data by asserting it matches exactly.
    UnData,
    // TODO: make unlabel take the label index as an arg.
    /// Destructures a label.
    UnLabel,
    /// Destructures a tuple.
    UnTuple,
    /// Add two numbers on the stack.
    Add,
    /// Subtract two numbers on the stack.
    Sub,
    /// Negate a number.
    Neg,
    /// Multiple two numbers on the stack.
    Mul,
    /// Divide two numbers, raising ZeroDiv side effect
    Div,
    /// Take the remainder of two numbers, raising ZeroDiv side effect
    Rem,
    /// Take a number to a power.
    Pow,
    /// Does nothing. Must always be last.
    Noop,
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
