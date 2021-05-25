use std::fmt;

use crate::common::{
    opcode::Opcode,
    data::Data,
    number::build_number,
    span::Span,
};

use crate::core::ffi::FFIFunction;

/// Represents a variable visible in the current scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    /// The index on the stack if the variable is local to the current scope.
    Local(usize),
    /// The index of the upvalue in the enclosing scope.
    Nonlocal(usize),
}

/// Represents a single interpretable chunk of bytecode,
/// think a function.
#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    // TODO: make this a list of variable names
    // So structs can be made, and state preserved in the repl.
    /// Number of variables declared in this scope.
    pub decls: usize,
    /// Each byte is an opcode or a number-stream.
    pub code: Vec<u8>,
    /// Each usize indexes the bytecode op that begins each line.
    pub spans: Vec<(usize, Span)>,
    /// Number-stream indexed, used to load constants.
    pub constants: Vec<Data>,
    /// List of positions of locals in the scope where this lambda is defined,
    /// indexes must be gauranteed to be data on the heap.
    pub captures: Vec<Captured>,
    /// List of FFI functions (i.e. Rust functions)
    /// that can be called from this function.
    pub ffi: Vec<FFIFunction>,
}

impl Lambda {
    /// Creates a new empty `Lambda` to be filled.
    pub fn empty() -> Lambda {
        Lambda {
            decls:     0,
            code:      vec![],
            spans:     vec![],
            constants: vec![],
            captures:  vec![],
            ffi:       vec![],
        }
    }

    /// Constructs a number of bytecode arguments,
    /// ensuring each is within a specific bound.
    /// If any bounds are violated, we return `None`.
    pub fn args_safe(&self, index: usize, within: &[usize]) -> Option<(Vec<usize>, usize)> {
        let mut offset  = 0;
        let mut numbers = vec![];

        for bound in within.iter() {
            let (arg, consumed) = build_number(&self.code[index + offset..]);
            if arg >= *bound { return None; }
            numbers.push(arg);
            offset += consumed;
        }

        return Some((numbers, offset));
    }

    pub fn bounds(&self, opcode: Opcode) -> Vec<usize> {
        match opcode {
            Opcode::Con     => vec![self.constants.len()],
            Opcode::NotInit => vec![],
            Opcode::Del     => vec![],
            Opcode::FFICall => vec![self.ffi.len()],
            Opcode::Copy    => vec![],
            Opcode::Capture => vec![self.decls], // TODO: correct bounds check?
            Opcode::Save    => vec![self.decls],
            Opcode::SaveCap => vec![self.captures.len()],
            Opcode::Load    => vec![self.decls],
            Opcode::LoadCap => vec![self.captures.len()],
            Opcode::Call    => vec![],
            Opcode::Return  => vec![self.decls], // TODO: correct bounds check?
            Opcode::Closure => vec![],
            Opcode::Print   => vec![],
            Opcode::Label   => vec![],
            Opcode::Tuple   => vec![usize::MAX], // TODO: stricter bounds
            Opcode::UnData  => vec![],
            Opcode::UnLabel => vec![],
            Opcode::UnTuple => vec![usize::MAX], // TODO: stricter bounds
            Opcode::Noop    => vec![],
        }
    }

    /// NOTE: WIP, do not use.
    /// Statically verifies some bytecode safely,
    /// By ensuring bytecode ops are within bounds,
    /// As well as the arguments those ops take.
    /// Returns `false` if the bytecode is invalid.
    pub fn verify(&self) -> bool {
        // go through each opcode
        // check the number of arguments
        // check that the arguments are valid
        let mut index = 0;

        while index < self.code.len() {
            // safely decode an opcode
            let opcode = match Opcode::from_byte_safe(self.code[index]) {
                Some(o) => o,
                None => { return false; },
            };

            index += 1;
            let bounds = self.bounds(opcode);
            let args_result = self.args_safe(index, &bounds);

            index += match args_result {
                Some((_args, consumed)) => consumed,
                None => { return false; },
            }
        }

        return true;
    }

    /// Emits an opcode as a byte.
    pub fn emit(&mut self, op: Opcode) {
        self.code.push(op as u8)
    }

    /// Emits a series of bytes.
    pub fn emit_bytes(&mut self, bytes: &mut Vec<u8>) {
        self.code.append(bytes)
    }

    /// Emits a span, should be called before an opcode is emmited.
    /// This function ties opcodes to spans in source.
    /// See index_span as well.
    pub fn emit_span(&mut self, span: &Span) {
        self.spans.push((self.code.len(), span.clone()))
    }

    /// Removes the last emitted byte.
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
    pub fn index_span(&self, index: usize) -> Span {
        let mut best = &Span::empty();

        for (i, span) in self.spans.iter() {
            if i > &index { break; }
            best = span;
        }

        return best.clone();
    }

    /// Adds a ffi function to the ffi table,
    /// without checking for duplicates.
    /// The `Compiler` ensures that functions are valid
    /// and not duplicated during codegen.
    pub fn add_ffi(&mut self, function: FFIFunction) -> usize {
        self.ffi.push(function);
        self.ffi.len() - 1
    }
}

impl fmt::Display for Lambda {
    /// Dump a human-readable breakdown of a `Lambda`'s bytecode.
    /// Including constants, captures, and variables declared.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Dumping Constants:")?;
        for constant in self.constants.iter() {
            writeln!(f, "{:?}", constant)?;
        }

        writeln!(f, "Dumping Captures:")?;
        for capture in self.captures.iter() {
            writeln!(f, "{:?}", capture)?;
        }

        writeln!(f, "Dumping Variables: {}", self.decls)?;

        writeln!(f, "Dumping Bytecode:")?;
        writeln!(f, "Inst    \tArgs")?;

        let mut index = 0;

        while index < self.code.len() {
            // safely decode an opcode
            let opcode = match Opcode::from_byte_safe(self.code[index]) {
                Some(o) => o,
                None => {
                    writeln!(f, "Invalid Opcode at index {}", index)?;
                    break;
                },
            };

            write!(f, "{:8?}", opcode)?;

            index += 1;
            let bounds = self.bounds(opcode);
            let args_result = self.args_safe(index, &bounds);

            let (args, consumed) = match args_result {
                Some((a, c)) => (a, c),
                None => {
                    writeln!(f, "Invalid Opcode argument at index {}", index)?;
                    break;
                },
            };

            index += consumed;
            writeln!(
                f, "{}",
                args.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join("\t")
            )?;
        }

        return fmt::Result::Ok(());
    }
}
