use crate::common::{
    opcode::Opcode,
    data::Data,
    local::Local,
    number::build_number,
};
use std::{
    fmt,
    io::Write,
    mem,
};

// // TODO: annotations in bytecode
// // TODO: separate AST compiler from Lambda itself
//
// /// The bytecode generator (emitter) walks the AST and produces (unoptimized) Bytecode
// /// There are plans to add a bytecode optimizer in the future.
// pub fn gen(ast: Spanned<AST>) -> Lambda {
//     let mut generator = Lambda::empty();
//     generator.walk(&ast);
//     generator
// }

/// Represents a single interpretable chunk of bytecode,
/// Think a function.
#[derive(Clone, Eq, PartialEq)]
pub struct Lambda {
    pub code:      Vec<u8>,    // each byte is an opcode or a number-stream
    pub offsets:   Vec<usize>, // each usize indexes the bytecode op that begins each line
    pub constants: Vec<Data>,  // number-stream indexed, used to load constants
}

impl Lambda {
    /// Creates a new empty Lambda to be filled.
    pub fn empty() -> Lambda {
        Lambda {
            code:      vec![],
            offsets:   vec![],
            constants: vec![],
        }
    }

    /// Emits an opcode as a byte
    pub fn emit(&mut self, op: Opcode) {
        self.code.push(op as u8)
    }

    /// Emits a series of bytes
    pub fn emit_bytes(&mut self, bytes: &mut Vec<u8>) {
        self.code.append(bytes)
    }

    /// Removes the last emitted byte
    pub fn demit(&mut self) {
        self.code.pop();
    }

    /// Given some data, this function adds it to the constants table,
    /// and returns the data's index.
    /// The constants table is push only, so constants are identified by their index.
    /// The resulting usize can be split up into a number byte stream,
    /// and be inserted into the bytecode.
    pub fn index_data(&mut self, data: Data) -> usize {
        match self.constants.iter().position(|d| d == &data) {
            Some(d) => d,
            None => {
                self.constants.push(data);
                self.constants.len() - 1
            },
        }
    }
}

impl fmt::Debug for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n-- Dumping Constants:")?;
        for constant in self.constants.iter() {
            writeln!(f, "{:?}", constant)?;
        }
        writeln!(f, "-- Dumping Bytecode:")?;
        writeln!(f, "Inst.   \tArg?\tValue?")?;
        let mut index = 0;

        while index < self.code.len() {
            index += 1;
            match Opcode::from_byte(self.code[index - 1]) {
                Opcode::Con => {
                    let (constant_index, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Load Con\t{}\t{:?}", constant_index, self.constants[constant_index])?;
                },
                Opcode::Del => { writeln!(f, "Delete  \t--\t--")?; },
                Opcode::Capture => { writeln!(f, "Capture \t--\tHeapify top of stack")?; },
                Opcode::Save => {
                    let (local_index, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Save    \t{}\tIndexed local", local_index)?;
                },
                Opcode::SaveCap => {
                    let (upvalue_index, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Save Cap\t{}\tIndexed upvalue on heap", upvalue_index)?;
                },
                Opcode::Load => {
                    let (local_index, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Load    \t{}\tIndexed local", local_index)?;
                },
                Opcode::LoadCap => {
                    let (upvalue_index, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Load Cap\t{}\tIndexed upvalue on heap", upvalue_index)?;
                },
                Opcode::Call => { writeln!(f, "Call    \t--\tRun top function using next stack value")?; }
                Opcode::Return => {
                    let (num_locals, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Return  \t{}\tLocals on stack deleted", num_locals)?;
                }
            }
        }

        Ok(())
    }
}

impl Debug for Lambda {

}
