use crate::Pointer;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    // Unsigned 64-bit natural numbers
    NatAdd,
    NatSub,
    NatMul,
    NatDiv,
    NatShl,
    NatShr,
    NatBitAnd,
    NatBitOr,
    NatBitXor,
    NatEq,
    NatToFloat,

    // Signed 64-bit integers
    IntAdd,
    IntSub,
    IntMul,
    IntDiv,
    IntEq,
    IntToFloat,

    // Signed 64-bit floating point numbers
    // NaNs are not allowed
    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatPartialEq,

    // Boolean
    BoolAnd,
    BoolOr,
    BoolXor,
    BoolNot,
    BoolEq,

    // Vaporization
    AllocSingle, // R1 _    -> owned pointer
    AllocPair,   // R1 R2   -> owned pointer
    AllocMany,   // SIZE _  -> owned pointer
    Borrow,      // PTR _   -> borrowed pointer
    ReadSingle,  // PTR IDX      -> R3
    ReadMany,    // PTR IDX SIZE -> onto stack
    WriteOwned,  // owned_pointer idx val
    Write,       // PTR SIZE -> R3 owned pointer / (loc of copying code on stack)

    // Pointer
    PointerIsOwned,    // PTR _ -> bool
    PointerIsBorrowed, // PTR _ -> bool

    // Between Stack and register
    PushSingle, // R1 _ -> onto stack
    PushPair,   // R1 R2 -> onto stack
    PushTriple, // R1 R2 R3 -> onto stack
    PopSingle,  // R1
    PopPair,    // R1 R2
    PopTriple,  // R1 R2 R3

    // Stack
    DelSingle,  // R1
    DelPair,    // R1 R2
    DelTriple,  // R1 R2 R3
    StackConst, // Idx

    // Register
    RegConst, // Idx _ -> R3
    RegSwap,  // R1 _ -> R3

    // Control Flow
    JumpTrue,   // loc R1 -> _
    JumpFalse,  // loc R1 -> _
    JumpBranch, // loc1 R1 loc2
    Call,       // code num_args num_captures
    Return,     // R1 _ -> _
    ReturnMany, // R1 num_ret -> _

    // Higher Order
    ClosureSingle, // loc R2           -> owned pointer to closure
    Closure,       // loc num_captures -> owned pointer to closure

    // Fibers and Handlers
    Fiber,       // code num_captures -> owned pointer to fiber
    FiberCall,   // arg fiber -> R3
    FiberYield,  // arg _ -> R3
    HandlerAdd,  // effect fiber -> _
    HandlerCall, // effect -> R3
}

#[derive(Debug, Clone)]
pub struct Instr {
    op_code: OpCode,
    r0: u8,
    r1: u8,
    out: u8,
}

impl Instr {
    pub fn exec(&self, regs: &mut [u64]) -> () {
        let r0  = self.r0 as usize;
        let r1  = self.r1 as usize;
        let out = self.out as usize;

        use std::mem::transmute as tm;
        use OpCode::*;

        // SAFETY: arguments gauranteed to be bitwise representation of ints: size(i64) == size(u64)
        let int_binop = |f: fn(i64, i64) -> i64, a0: u64, a1: u64| unsafe {
            let b0: i64 = tm(a0);
            let b1: i64 = tm(a1);
            tm(f(b0, b1))
        };

        // SAFETY: arguments gauranteed to be bitwise representation of floats: size(f64) == size(u64)
        let float_binop = |f: fn(f64, f64) -> f64, a0: u64, a1: u64| unsafe {
            let b0: f64 = tm(a0);
            let b1: f64 = tm(a1);
            tm(f(b0, b1))
        };

        match self.op_code {
            NatAdd => { regs[out] = regs[r0] + regs[r1] },
            NatSub => { regs[out] = regs[r0] - regs[r1] },
            NatMul => { regs[out] = regs[r0] * regs[r1] },
            NatDiv => { regs[out] = regs[r0] / regs[r1] },
            NatShl => { regs[out] = regs[r0] << regs[r1] },
            NatShr => { regs[out] = regs[r0] >> regs[r1] },
            NatBitAnd => { regs[out] = regs[r0] & regs[r1] },
            NatBitOr  => { regs[out] = regs[r0] | regs[r1] },
            NatBitXor => { regs[out] = regs[r0] ^ regs[r1] },
            NatEq     => { todo!("booleans!"); },

            // Signed 64-bit integers
            IntAdd => { regs[out] = int_binop(|a, b| { a + b }, regs[r0], regs[r1]) },
            IntSub => { regs[out] = int_binop(|a, b| { a - b }, regs[r0], regs[r1]) },
            IntMul => { regs[out] = int_binop(|a, b| { a * b }, regs[r0], regs[r1]) },
            IntDiv => { regs[out] = int_binop(|a, b| { a / b }, regs[r0], regs[r1]) },
            IntEq  => { todo!("booleans!") },

            // Signed 64-bit floating point numbers
            // NaNs are not allowed
            FloatAdd => { regs[out] = float_binop(|a, b| { a + b }, regs[r0], regs[r1]) },
            FloatSub => { regs[out] = float_binop(|a, b| { a - b }, regs[r0], regs[r1]) },
            FloatMul => { regs[out] = float_binop(|a, b| { a * b }, regs[r0], regs[r1]) },
            FloatDiv => { regs[out] = float_binop(|a, b| { a / b }, regs[r0], regs[r1]) },
            FloatPartialEq => { todo!("booleans!") },

            // Boolean
            | BoolAnd
            | BoolOr
            | BoolXor
            | BoolNot
            | BoolEq => { todo!("booleans!") },

            _ => todo!("not yep implemented"),

            // // Vaporization
            // AllocSingle, // R1 _    -> owned pointer
            // AllocPair,   // R1 R2   -> owned pointer
            // AllocMany,   // SIZE _  -> owned pointer
            // Borrow,      // PTR _   -> borrowed pointer
            // ReadSingle,  // PTR IDX      -> R3
            // ReadMany,    // PTR IDX SIZE -> onto stack
            // WriteSingle, // PTR IDX VAL  -> copy on write
            // WriteMany,   // PTR IDX SIZE -> from stack
            //
            // // Pointer
            PointerIsOwned => { },
            // PointerIsBorrowed, // PTR _ -> bool
            //
            // // Between Stack and register
            // PushSingle, // R1 _ -> onto stack
            // PushPair,   // R1 R2 -> onto stack
            // PushTriple, // R1 R2 R3 -> onto stack
            // PopSingle,  // R1
            // PopPair,    // R1 R2
            // PopTriple,  // R1 R2 R3
            //
            // // Stack
            // DelSingle,  // R1
            // DelPair,    // R1 R2
            // DelTriple,  // R1 R2 R3
            // StackConst, // Idx
            //
            // // Register
            // RegConst, // Idx _ -> R3
            // RegSwap,  // R1 _ -> R3
            //
            // // Control Flow
            // JumpTrue,   // loc R1 -> _
            // JumpFalse,  // loc R1 -> _
            // Call,       // code num_args num_captures
            // Return,     // R1 _ -> _
            // ReturnMany, // R1 num_ret -> _
            //
            // // Higher Order
            // ClosureSingle, // loc R2           -> owned pointer to closure
            // Closure,       // loc num_captures -> owned pointer to closure
            //
            // // Fibers and Handlers
            // Fiber,       // code num_captures -> owned pointer to fiber
            // FiberCall,   // arg fiber -> R3
            // FiberYield,  // arg _ -> R3
            // HandlerAdd,  // effect fiber -> _
            // HandlerCall, // effect -> R3
        }
        todo!()
    }
}
