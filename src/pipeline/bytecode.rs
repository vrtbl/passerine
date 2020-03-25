use crate::utils::annotation::Annotation;
use crate::vm::data::Data;

// the bytecode needs to contain annotation information as well

pub type Bytecode = Vec<u8>;
pub type Constants = Vec<Data>;

pub enum Opcode {
//  Opcode // operands, top of stack first
    Const, // index as a chain of bytes -> pushes value from constant table onto stack
    Save,  // Data, Symbol -> Stores Data in Symbol
    Load,  // Symbol -> replaces symbol on top of stack with its value
    Clear, // clears stack to last frame
}

// TODO: debug bytecode display function... utils, perhaps?

impl Opcode {
    // to_byte and from_byte are opposites...
    // macro time?

    pub fn to_byte(&self) -> u8 {
        match self {
            Opcode::Const => 0,
            Opcode::Save  => 1,
            Opcode::Load  => 2,
            Opcode::Clear => 3,
        }
    }

    pub fn same(&self, byte: u8) -> bool {
        byte == self.to_byte()
    }
}

// No tests for now, they're just type aliases
