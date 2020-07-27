use std::mem;

use crate::utils::number::build_number;
use crate::utils::runtime::Trace;
use crate::vm::local::Local;
use crate::vm::data::{Data, Tagged};
use crate::vm::stack::{Stack, Item};
use crate::pipeline::bytecode::Opcode;
use crate::compiler::gen::Chunk; // TODO: Move chunk to a better spot?

/// A `VM` executes bytecode chunks.
/// Each VM's state is self-contained,
/// So more than one can be spawned if needed.
#[derive(Debug)]
pub struct VM {
    chunk: Chunk,
    stack: Stack,
    ip:    usize,
}

// NOTE: use Opcode::same and Opcode.to_byte() rather than actual bytes
// Don't worry, the compiler *should* get rid of this overhead and just use bytes

// this impl contains initialization, helper functions, and the core interpreter loop
// the next impl contains opcode implementations
impl VM {
    /// Initialize a new VM.
    /// To run the VM, a chunk must be passed to it through `run`.
    pub fn init() -> VM {
        VM {
            chunk: Chunk::empty(),
            stack: vec![Item::Frame],
            ip:    0,
        }
    }


    fn next(&mut self)                           { self.ip += 1; }
    fn terminate(&mut self) -> Result<(), Trace> { self.ip = self.chunk.code.len(); Ok(()) }
    fn done(&mut self)      -> Result<(), Trace> { self.next(); Ok(()) }
    fn peek_byte(&mut self) -> u8                { self.chunk.code[self.ip] }
    fn next_byte(&mut self) -> u8                { self.done(); self.peek_byte() }

    /// Builds the next number in the bytecode stream.
    /// See `utils::number` for more.
    fn next_number(&mut self) -> usize {
        self.next();
        let remaining      = self.chunk.code[self.ip..].to_vec();
        let (index, eaten) = build_number(remaining);
        self.ip += eaten - 1; // ip left on next op
        return index;
    }

    /// Finds a local on stack.
    /// Note that in the future, locals should be pre-indexed when the AST is walked
    /// so this function won't be necessary then.
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

    /// finds the index of the local on the stack indicated by the bytecode.
    fn local_index(&mut self) -> (Local, Option<usize>) {
        let local_index = self.next_number();
        let local       = self.chunk.locals[local_index].clone();
        let index       = self.find_local(&local);

        return (local, index);
    }

    // core interpreter loop

    /// Dissasembles and interprets a single (potentially fallible) bytecode op.
    /// The op definitions follow in the proceeding impl block.
    /// To see what each op does, check `pipeline::bytecode.rs`
    fn step(&mut self) -> Result<(), Trace> {
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

    /// Suspends the current chunk and runs a new one on the VM.
    /// Runs until either success, in which it restores the state of the previous chunk,
    /// Or failure, in which it returns the runtime error.
    /// In the future, fibers will allow for error handling -
    /// right now, error in Passerine are practically panics.
    fn run(&mut self, chunk: Chunk) -> Result<(), Trace> {
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
        return Result::Ok(());
    }
}

// TODO: there are a lot of optimizations that can be made
// I'll list a few here:
// - searching the stack for variables
//   A global Hash-table has significantly less overhead for function calls
// - cloning the heck out of everything - useless copies
//   instead, lifetime analysis during compilation
// - replace some panics with Result<()>s
impl VM {
    /// Load a constant and push it onto the stack.
    fn con(&mut self) -> Result<(), Trace> {
        // get the constant index
        let index = self.next_number();

        self.stack.push(Item::Data(Tagged::from(self.chunk.constants[index].clone())));
        self.done()
    }

    /// Save the topmost value on the stack into a variable.
    fn save(&mut self) -> Result<(), Trace> {
        let data = match self.stack.pop() { Some(Item::Data(d)) => d.data(), _ => panic!("Expected data") };
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

    /// Push a copy of a variable's value onto the stack.
    fn load(&mut self) -> Result<(), Trace> {
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

    /// Clear all data off the stack.
    fn clear(&mut self) -> Result<(), Trace> {
        loop {
            match self.stack.pop() {
                Some(Item::Data(_)) => (),
                Some(l)             => { self.stack.push(l); break; },
                None                => panic!("There wasn't a frame on the stack.")
            }
        }

        self.done()
    }

    /// Call a function on the top of the stack, passing the next value as an argument.
    fn call(&mut self) -> Result<(), Trace> {
        let fun = match self.stack.pop() {
            Some(Item::Data(d)) => match d.data() {
                Data::Lambda(l) => l,
                _               => unreachable!(),
            }
            _ => unreachable!(),
        };
        let arg = match self.stack.pop() {
            Some(Item::Data(d)) => d,
            _                   => unreachable!(),
        };

        self.stack.push(Item::Frame);
        self.stack.push(Item::Data(arg));
        // println!("entering...");
        self.run(fun);
        // println!("exiting...");

        self.done()
    }

    /// Return a value from a function.
    /// End the execution of the current chunk.
    /// Relpaces the last frame with the value on the top of the stack.
    fn return_val(&mut self) -> Result<(), Trace> {
        let val = match self.stack.pop() {
            Some(Item::Data(d)) => d,
            _                   => unreachable!(),
        };

        loop {
            match self.stack.pop() {
                Some(Item::Frame) => break,
                None              => unreachable!("There should never not be a frame on the stack"),
                _                 => (),
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
    use crate::pipeline::source::Source;

    #[test]
    fn init_run() {
        // TODO: check @ each step, write more tests
        let chunk = gen(parse(lex(
            Source::source("boop = 37.201; true; dhuiew = true; boop")
        ).unwrap()).unwrap());

        let mut vm = VM::init();

        match vm.run(chunk) {
            Result::Ok(_)      => (),
            Result::Trace(..)  => panic!("VM threw error."),
            Result::Syntax(..) => unreachable!(),
        }
    }

    #[test]
    fn block_expression() {
        let chunk = gen(parse(lex(
            Source::source("boop = true; heck = { x = boop; x }; heck")
        ).unwrap()).unwrap());

        let mut vm = VM::init();

        match vm.run(chunk) {
            Result::Ok(_)      => (),
            Result::Trace(..)  => panic!("VM threw error"),
            Result::Syntax(..) => unreachable!(),
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
            Source::source("iden = x -> x; y = true; iden ((iden iden) (iden y))")
        ).unwrap()).unwrap());

        let mut vm = VM::init();
        vm.run(chunk);

        if let Some(Item::Data(t)) = vm.stack.pop() {
            assert_eq!(t.data(), Data::Boolean(true));
        } else {
            panic!("Expected float on top of stack");
        }
    }

    #[test]
    fn fun_scope() {
        let chunk = gen(parse(lex(
            Source::source("y = (x -> { z = x; z }) 7.0; y")
        ).unwrap()).unwrap());

        let out_of_scope = Local::new("z".to_string());

        let mut vm = VM::init();
        vm.run(chunk);

        // check that z has been dealloced
        assert_eq!(vm.find_local(&out_of_scope), None);

        // check that y is in fact 7
        if let Some(Item::Data(t)) = vm.stack.pop() {
            assert_eq!(t.data(), Data::Real(7.0));
        } else {
            panic!("Expected 7.0 on top of stack");
        }
    }
}
