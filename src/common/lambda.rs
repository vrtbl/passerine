use crate::common::{
    opcode::Opcode,
    data::Data,
    number::build_number,
    span::Span,
};

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    /// The index on the stack if the variable is local to the current scope
    Local(usize),
    /// The index of the upvalue in the enclosing scope
    Nonlocal(usize),
}

/// Represents a single interpretable chunk of bytecode,
/// Think a function.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Lambda {
    /// Each byte is an opcode or a number-stream.
    pub code: Vec<u8>,
    /// Each usize indexes the bytecode op that begins each line.
    pub spans: Vec<(usize, Span)>,
    /// number-stream indexed, used to load constants.
    pub constants: Vec<Data>,
    /// List of positions of locals in the scope where this lambda is defined,
    /// indexes must be gauranteed to be data on the heap.
    pub captures: Vec<Captured>,

}

impl Lambda {
    /// Creates a new empty Lambda to be filled.
    pub fn empty() -> Lambda {
        Lambda {
            code:      vec![],
            spans:     vec![],
            constants: vec![],
            captures:    vec![],
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

    /// Emits a span, should be called before an opcode is emmited.
    /// This function ties opcodes to spans in source.
    /// See index_span as well.
    pub fn emit_span(&mut self, span: &Span) {
        self.spans.push((self.code.len(), span.clone()))
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

    /// Look up the nearest span at or before the index of a specific bytecode op.
    pub fn index_span(&mut self, index: usize) -> Span {
        let mut best = &Span::empty();

        for (i, span) in self.spans.iter() {
            if i > &index { break; }
            best = span;
        }

        return best.clone();
    }
}

impl fmt::Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n-- Dumping Constants:")?;
        for constant in self.constants.iter() {
            writeln!(f, "{:?}", constant)?;
        }

        // writeln!(f, "-- Dumping Spans:")?;
        // for span in self.spans.iter() {
        //     writeln!(f, "{:?}", span)?;
        // }

        writeln!(f, "-- Dumping Captures:")?;
        for capture in self.captures.iter() {
            writeln!(f, "{:?}", capture)?;
        }

        writeln!(f, "-- Dumping Bytecode:")?;
        writeln!(f, "Inst.   \tArgs\tValue?")?;
        let mut index = 0;

        while index < self.code.len() {
            index += 1;
            match Opcode::from_byte(self.code[index - 1]) {
                Opcode::Con => {
                    let (constant_index, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Load Con\t{}\t{:?}", constant_index, self.constants[constant_index])?;
                },
                Opcode::Del => { writeln!(f, "Delete  \t\t--")?; },
                Opcode::Capture => {
                    let (local_index, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Capture \t{}\tIndexed local moved to heap", local_index)?;
                },
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
                Opcode::Call => { writeln!(f, "Call    \t\tRun top function using next stack value")?; }
                Opcode::Return => {
                    let (num_locals, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Return  \t{}\tLocals on stack deleted", num_locals)?;
                },
                Opcode::Closure => {
                    let (todo_index, consumed) = build_number(&self.code[index..]);
                    index += consumed;
                    writeln!(f, "Closure \t{}\tIndex of lambda to be wrapped", todo_index)?;
                },
                Opcode::Print   => { writeln!(f, "Print    \t\t--")?; },
                Opcode::Label   => { writeln!(f, "Label    \t\t--")?; },
                Opcode::UnLabel => { writeln!(f, "UnLabel  \t\t--")?; },
                Opcode::UnData  => { writeln!(f, "UnData   \t\t--")?; },
                Opcode::Copy    => { writeln!(f, "Copy     \t\t--")?; },
            }
        }

        Ok(())
    }
}
