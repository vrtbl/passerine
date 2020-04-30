use std::mem;

use crate::utils::number::build_number;
use crate::vm::local::Local;
use crate::vm::data::{Data, Tagged};
use crate::vm::stack::{Stack, Item};
use crate::pipeline::bytecode::Opcode;
use crate::compiler::gen::Chunk; // Move chunk to a better spot?

// I'm not sure if a garbage collector is necessary
// Rust makes sure there are no memory leaks
// and all non-returned values are freed when they go out of scope as per design
// also, I'm cloning everything all over the place
// I need to either implement resiliant-whatever datastructures (like FP)
// or get my act together and do pass by object reference or something

#[derive(Debug)]
pub struct VM {
    chunk: Chunk,
    stack: Stack,
    ip:    usize,
}

type RunResult = Option<()>;

// NOTE: use Opcode::same and Opcode.to_byte() rather than actual bytes
// Don't worry, the compiler *should* get rid of this overhead and just use bytes

// this impl contains initialization, helper functions, and the core interpreter loop
// the below impl contains opcode implementations
impl VM {
    pub fn init() -> VM {
        VM {
            chunk: Chunk::empty(),
            stack: vec![Item::Frame],
            ip:    0,
        }
    }

    fn next(&mut self)                   { self.ip += 1; }
    fn terminate(&mut self) -> RunResult { self.ip = self.chunk.code.len(); Some(()) }
    fn done(&mut self)      -> RunResult { self.next(); Some(()) }
    fn peek_byte(&mut self) -> u8        { self.chunk.code[self.ip] }
    fn next_byte(&mut self) -> u8        { self.done(); self.peek_byte() }

    fn next_number(&mut self) -> usize {
        self.next();
        let remaining      = self.chunk.code[self.ip..].to_vec();
        let (index, eaten) = build_number(remaining);
        self.ip += eaten - 1; // ip left on next op
        return index;
    }

    fn find_local(&mut self, local: &Local) -> Option<usize> {
        for (index, item) in self.stack.iter().enumerate().rev() {
            match item {
                Item::Local { local: l, .. } => if local == l { return Some(index); },
                Item::Frame                  => break,
                Item::Data(_)                => (),
            }
        }

        return None;
    }

    fn local_index(&mut self) -> (Local, Option<usize>) {
        let local_index = self.next_number();
        let local       = self.chunk.locals[local_index].clone();
        let index       = self.find_local(&local);

        return (local, index);
    }

    // core interpreter loop

    fn step(&mut self) -> RunResult {
        let op_code = Opcode::from_byte(self.peek_byte());

        // println!("op_code: {}", self.peek_byte());
        match op_code {
            Opcode::Con    => self.con(),
            Opcode::Save   => self.save(),
            Opcode::Load   => self.load(),
            Opcode::Clear  => self.clear(),
            Opcode::Call   => self.call(),
            Opcode::Return => self.return_val(),
        }
    }

    fn run(&mut self, chunk: Chunk) -> RunResult {
        // cache current state, load new bytecode
        let old_chunk = mem::replace(&mut self.chunk, chunk);
        let old_ip    = mem::replace(&mut self.ip,    0);
        // TODO: should chunks store their own ip?

        while self.ip < self.chunk.code.len() {
            // println!("before: {:?}", self.stack);
            self.step();
        }

        // return current state
        mem::drop(mem::replace(&mut self.chunk, old_chunk));
        self.ip = old_ip;

        // nothing went wrong!
        return Some(());
    }
}

// TODO: there are a lot of optimizations that can be made
// I'll list a few here:
// - searching the stack for variables
//   A global Hash-table has significantly less overhead for function calls
// - cloning the heck out of everything - useless copies
// - replace some panics with runresults
impl VM {
    fn con(&mut self) -> RunResult {
        // get the constant index
        let index = self.next_number();

        self.stack.push(Item::Data(Tagged::from(self.chunk.constants[index].clone())));
        self.done()
    }

    fn save(&mut self) -> RunResult {
        let data = match self.stack.pop()? { Item::Data(d) => d.data(), _ => panic!("Expected data") };
        let (local, index) = self.local_index();

        // NOTE: Does it make a copy or does it make a reference?
        // It makes a copy of the data
        match index {
            // It's been declared
            Some(i) => mem::drop(
                mem::replace(
                    &mut self.stack[i],
                    Item::Local { local, data },
                )
            ),
            // It hasn't been declared
            None => self.stack.push(Item::Local { local, data }),
        }

        // TODO: make separate byte code op?
        self.stack.push(Item::Data(Tagged::from(Data::Unit)));

        self.done()
    }

    fn load(&mut self) -> RunResult {
        let (_, index) = self.local_index();

        match index {
            Some(i) => {
                if let Item::Local { data, .. } = &self.stack[i] {
                    let data = Item::Data(Tagged::from(data.clone()));
                    self.stack.push(data);
                }
            },
            None => panic!("Local not found on stack!"), // TODO: make it a Passerine error
        }

        self.done()
    }

    fn clear(&mut self) -> RunResult {
        loop {
            match self.stack.pop()? {
                Item::Data(_) => (),
                l             => { self.stack.push(l); break; },
            }
        }

        self.done()
    }

    fn call(&mut self) -> RunResult {
        let arg = match self.stack.pop()? {
            Item::Data(d) => d,
            _             => unreachable!(),
        };
        let fun = match self.stack.pop()? {
            Item::Data(d) => match d.data() {
                Data::Lambda(l) => l,
                _               => unreachable!(),
            }
            _ => unreachable!(),
        };

        self.stack.push(Item::Frame);
        self.stack.push(Item::Data(arg));
        // println!("entering...");
        self.run(fun);
        // println!("exiting...");

        self.done()
    }

    fn return_val(&mut self) -> RunResult {
        let val = match self.stack.pop()? {
            Item::Data(d) => d,
            _             => unreachable!(),
        };

        loop {
            // TODO: panic if no frames on stack?
            if let Item::Frame = self.stack.pop()? {
                break;
            }
        }

        self.stack.push(Item::Data(val));
        self.terminate()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::{
        parse::parse,
        lex::lex,
        gen::gen,
    };

    #[test]
    fn init_run() {
        // TODO: check @ each step, write more tests
        let chunk = gen(parse(lex(
            "boop = 37.201; true; dhuiew = true; boop"
        ).unwrap()).unwrap());

        let mut vm = VM::init();

        match vm.run(chunk) {
            Some(_) => (),
            None    => panic!("VM threw error"),
        }
    }

    #[test]
    fn block_expression() {
        let chunk = gen(parse(lex(
            "boop = true; heck = { x = boop; x }; heck"
        ).unwrap()).unwrap());

        let mut vm = VM::init();

        match vm.run(chunk) {
            Some(_) => (),
            None    => panic!("VM threw error"),
        }

        if let Some(Item::Data(t)) = vm.stack.pop() {
            match t.data() {
                Data::Boolean(true) => (),
                _                   => panic!("Expected true value"),
            }
        } else {
            panic!("Expected data on top of stack");
        }
    }

    #[test]
    fn functions() {
        let chunk = gen(parse(lex(
            "iden = x -> x; y = true; iden ((iden iden) (iden y))"
        ).unwrap()).unwrap());

        let mut vm = VM::init();
        vm.run(chunk);

        if let Some(Item::Data(t)) = vm.stack.pop() {
            assert_eq!(t.data(), Data::Boolean(true));
        } else {
            panic!("Expected float on top of stack");
        }
    }

    fn fun_scope() {
        let chunk = gen(parse(lex(
            "iden = x -> x; y = true; iden ((iden iden) (iden y))"
        ).unwrap()).unwrap());
    }
}
