use crate::utils::annotation::Annotation;
use crate::vm::data::Data;

// the bytecode needs to contain annotation information as well

pub type Bytecode = Vec<u8>;
pub type Constants = Vec<Data>;

pub enum Opcode {
//  Opcode // operands, top of stack first
    Con,   // index as a chain of bytes -> pushes value from constant table onto stack
    Save,  // Data, Symbol -> Stores Data in Symbol
    Load,  // Symbol -> replaces symbol on top of stack with its value
    Clear, // clears stack to last frame
}

// TODO: debug bytecode display function... utils, perhaps?

impl Opcode {
    // to_byte and from_byte are opposites...
    // avoid duplication of knowledge...
    // macro time?

    pub fn to_byte(&self) -> u8 {
        match self {
            Opcode::Con   => 0,
            Opcode::Save  => 1,
            Opcode::Load  => 2,
            Opcode::Clear => 3,
        }
    }

    // TODO: just use macro to inverse match?
    pub fn to_op(byte: u8) -> Opcode {
        match byte {
            0 => Opcode::Con,
            1 => Opcode::Save,
            2 => Opcode::Load,
            3 => Opcode::Clear,
            _ => panic!("Unknown opcode"),
        }
    }

    pub fn same(&self, byte: u8) -> bool {
        byte == self.to_byte()
    }
}

// No tests for now, they're just type aliases
