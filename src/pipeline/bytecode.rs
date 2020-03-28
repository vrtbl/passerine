use crate::vm::data::Data;
use crate::vm::local::Local;

#[repr(u8)]
pub enum Opcode {
// § indicates location of op
// note: all args are before op, byte-stream args after
//  Opcode // operands (stack top first) § byte-streams -> Does
    Con   = 0,   // § byte-stream -> pushes value from constant table onto stack
    Save  = 1,  // Data § byte-stream -> Stores Data in Symbol
    Load  = 2,  // Symbol § -> replaces symbol on top of stack with its value
    Clear = 3, // § -> clears stack to last frame
}

impl Opcode {
    // This function was hard to write
    // after reading a bunch of rust specs,
    // I decided to just use some unsafe code instead of a huge if-else trees
    // gaurded match statements, or procedural macros
    // this is *very* unsafe, though
    pub fn from_byte(byte: u8) -> Opcode {
        let e: Opcode = unsafe { std::mem::transmute(byte) }; // *chuckles in undefined behavior*
        return e; // "I'm in danger"
    }
    // No, but it's just an enum
    // And this function should never fail
    // if it does, there's an obvious error with the bytecode generator or the vm
    // that needs to be addressed
    // If you have a better solution, or Rust changes so such a solution is more idiomatic
    // open an issue, fast!
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Chunk {
    pub code:      Vec<u8>,    // each byte is an opcode or a number-stream
    pub offsets:   Vec<usize>, // each usize indexes the bytecode op that begins each line
    pub constants: Vec<Data>,  // number-stream indexed, used to load constants
    pub locals:    Vec<Local>, // ''                                  variables
}

impl Chunk {
    pub fn empty() -> Chunk {
        Chunk {
            code:      vec![],
            offsets:   vec![],
            constants: vec![],
            locals:    vec![],
        }
    }

    pub fn index_constant(&mut self, data: Data) -> usize {
        match self.constants.iter().position(|d| d == &data) {
            Some(d) => d,
            None    => {
                self.constants.push(data.clone());
                self.constants.len() - 1
            },
        }
    }

    pub fn index_local(&mut self, local: Local) -> usize {
        match self.locals.iter().position(|l| l == &local) {
            Some(l) => l,
            None    => {
                self.locals.push(local);
                self.locals.len() - 1
            },
        }
    }

    // TODO: bytecode chunk dissambler
}
