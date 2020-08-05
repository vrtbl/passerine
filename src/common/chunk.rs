use crate::common::opcode::Opcode;
use crate::common::data::Data;
use crate::common::local::Local;

// // TODO: annotations in bytecode
// // TODO: separate AST compiler from Chunk itself
//
// /// The bytecode generator (emitter) walks the AST and produces (unoptimized) Bytecode
// /// There are plans to add a bytecode optimizer in the future.
// pub fn gen(ast: Spanned<AST>) -> Chunk {
//     let mut generator = Chunk::empty();
//     generator.walk(&ast);
//     generator
// }

/// Represents a single interpretable chunk of bytecode,
/// Think a function.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Chunk {
    pub code:      Vec<u8>,    // each byte is an opcode or a number-stream
    pub offsets:   Vec<usize>, // each usize indexes the bytecode op that begins each line
    pub constants: Vec<Data>,  // number-stream indexed, used to load constants
}

impl Chunk {
    /// Creates a new empty chunk to be filled.
    pub fn empty() -> Chunk {
        Chunk {
            code:      vec![],
            offsets:   vec![],
            constants: vec![],
        }
    }

    pub fn emit(&mut self, op: Opcode) {
        self.code.push(op as u8)
    }

    pub fn emit_bytes(&mut self, bytes: &mut Vec<u8>) {
        self.code.append(bytes)
    }


    /// Given some data, this function adds it to the constants table,
    /// and returns the data's index.
    /// The constants table is push only, so constants are identified by their index.
    /// The resulting usize can be split up into a number byte stream,
    /// and be inserted into the bytecode.
    pub fn index_data(&mut self, data: Data) -> usize {
        match self.constants.iter().position(|d| d == &data) {
            Some(d) => d,
            None    => {
                self.constants.push(data);
                self.constants.len() - 1
            },
        }
    }

    /// Similar to index constant, but indexes variables instead.
    fn index_symbol(&mut self, symbol: Local) -> usize {
        match self.locals.iter().position(|l| l == &symbol) {
            Some(l) => l,
            None    => {
                self.locals.push(symbol);
                self.locals.len() - 1
            },
        }
    }
}
