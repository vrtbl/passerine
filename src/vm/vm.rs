use crate::utils::number::build_number;
use std::mem;

use crate::vm::data::Data;
use crate::vm::stack::{Stack, Item};
use crate::pipeline::bytecode::{Bytecode, Constants, Opcode};

// I'm not sure if a garbage collector is necessary
// Rust makes sure there are no memory leaks
// and all non-returned values are freed when they go out of scope as per design
// also, I'm cloning everything all over the place
// I need to either implement resiliant-whatever datastructures (like FP)
// or get my act together and do pass by object reference or something

pub struct VM {
    constants: Constants,
    bytecode:  Bytecode,
    stack:     Stack,
    ip:        usize,
}

type RunResult = Option<()>;

// NOTE: use Opcode::same and Opcode.to_byte() rather than actual bytes
// Don't worry, the compiler *should* get rid of this overhead and just use bytes

// this impl contains initialization, helper functions, and the core interpreter loop
// the below impl contains opcode implementations
impl VM {
    pub fn init(constants: Constants) -> VM {
        VM {
            constants: constants, // just initialize constants here?
            bytecode:  vec![],
            stack:     vec![Item::frame()],
            ip:        0,
        }
    }

    fn next(&mut self)                   { self.ip += 1; }
    fn done(&mut self)      -> RunResult { self.next(); Some(()) }
    fn peek_byte(&mut self) -> u8        { self.bytecode[self.ip] }
    fn next_byte(&mut self) -> u8        { self.done(); self.peek_byte() }

    fn frame_indicies(&self) -> Vec<usize> {
        let mut indices = vec![];

        for (index, item) in self.stack.iter().enumerate() {
            if let Item::Frame(_) = item {
                indices.push(index);
            }
        }

        // flip from global -> local to local -> global
        indices.reverse();
        return indices;
    }

    // core interpreter loop

    fn step(&mut self) -> RunResult {
        let op_code = Opcode::to_op(self.peek_byte());

        match op_code {
            Opcode::Con   => self.con(),
            Opcode::Save  => self.save(),
            Opcode::Load  => self.load(),
            Opcode::Clear => self.clear(),
        }
    }

    fn run(&mut self, bytecode: Bytecode) -> RunResult {
        // cache current state, load new bytecode
        let old_ip = mem::replace(&mut self.ip,       0);
        let old_bc = mem::replace(&mut self.bytecode, bytecode);

        use std::time::Instant;
        let now = Instant::now();
        while self.ip < self.bytecode.len() {
            self.step();
        }
        let elapsed = now.elapsed();
        println!("Elapsed: {:?}", elapsed);

        // return current state
        mem::drop(mem::replace(&mut self.ip,       old_ip));
        mem::drop(mem::replace(&mut self.bytecode, old_bc));

        // nothing went wrong!
        return Some(());
    }
}

// TODO: there are a lot of optimizations that can be made
// i'll list a few here:
// - searching the stack for variables
//   A global Hash-table has significantly less overhead for function calls
// - cloning the heck out of everything - useless copies
// - replace some panics with runresults
impl VM {
    fn con(&mut self) -> RunResult {
        // get the constant index
        let remaining      = self.bytecode[self.ip..].to_vec();
        let (index, eaten) = build_number(remaining);

        // push constant onto stack
        self.ip += eaten - 1;
        self.stack.push(Item::data(self.constants[index].clone()));
        self.done()
    }

    fn save(&mut self) -> RunResult {
        let data   = match self.stack.pop()? { Item::Data(d)               => d, _ => panic!("Expected data")   };
        let symbol = match self.stack.pop()? { Item::Data(Data::Symbol(s)) => s, _ => panic!("Expected symbol") };

        let mut declared = false;

        // we go back through frames until we find the variable
        // if the vairable doesn't exist, we declare it in the current frame
        for index in self.frame_indicies() {
            if let Item::Frame(frame) = &mut self.stack[index] {
                if frame.contains_key(&symbol) {
                    frame.insert(symbol.clone(), data.clone());
                    declared = true;
                    break;
                }
            }
        }

        if !declared {
            let index = *self.frame_indicies().last()?;
            if let Item::Frame(frame) = &mut self.stack[index] {
                frame.insert(symbol, data);
            } else {
                panic!("No stack frames present")
            }
        }

        self.done()
    }

    fn load(&mut self) -> RunResult {
        let mut value = None;

        if let Item::Data(Data::Symbol(symbol)) = self.stack.pop()? {
            for index in self.frame_indicies() {
                if let Item::Frame(frame) = &self.stack[index] {
                    match frame.get(&symbol) {
                        None    => (),
                        Some(v) => { value = Some(v.clone()); break; },
                    }
                }
            }
        } else {
            panic!("Expected symbol; symbol wasn't on stack")
        }

        match value {
            Some(v) => self.stack.push(Item::data(v)),
            None    => return None, // symbol not found in scope
        }

        self.done()
    }

    fn clear(&mut self) -> RunResult {
        loop {
            if let Item::Frame(frame) = self.stack.pop()? {
                self.stack.push(Item::Frame(frame));
                break;
            }
        }

        self.done()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pipes::{
        parse::parse,
        lex::lex,
        gen::gen,
    };

    #[test]
    fn init_run() {
        // TODO: check @ each step, write more tests
        let (bytecode, constants) = gen(parse(lex(
            "boop = true; true; dhuiew = true; boop"
        ).unwrap()).unwrap());

        println!("{:?}\n{:?}", bytecode, constants);

        let mut vm = VM::init(constants);

        match vm.run(bytecode) {
            Some(_) => (),
            None    => panic!("VM threw error"),
        }
    }
}
