use std::mem;

use crate::common::{
    number::build_number,
    data::Data,
    opcode::Opcode,
    chunk::Chunk,
};

use crate::vm::{
    trace::Trace,
    tag::Tagged,
    stack::Stack,
};

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
            stack: Stack::init(),
            ip:    0,
        }
    }

    fn next(&mut self)                           { self.ip += 1; }
    fn terminate(&mut self) -> Result<(), Trace> { self.ip = self.chunk.code.len(); Ok(()) }
    fn done(&mut self)      -> Result<(), Trace> { self.next(); Ok(()) }
    fn peek_byte(&mut self) -> u8                { self.chunk.code[self.ip] }
    fn next_byte(&mut self) -> u8                { self.done().unwrap(); self.peek_byte() }

    /// Builds the next number in the bytecode stream.
    /// See `utils::number` for more.
    fn next_number(&mut self) -> usize {
        self.next();
        let remaining      = self.chunk.code[self.ip..].to_vec();
        let (index, eaten) = build_number(remaining);
        self.ip += eaten - 1; // ip left on next op
        println!("{}", index);
        return index;
    }

    // core interpreter loop

    /// Dissasembles and interprets a single (potentially fallible) bytecode op.
    /// The op definitions follow in the proceeding impl block.
    /// To see what each op does, check `pipeline::bytecode.rs`
    fn step(&mut self) -> Result<(), Trace> {
        let op_code = Opcode::from_byte(self.peek_byte());

        match op_code {
            Opcode::Con    => self.con(),
            Opcode::Del    => self.del(),
            Opcode::Save   => self.save(),
            Opcode::Load   => self.load(),
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
            println!("before: {:?}", self.stack);
            println!("executing: {:?}", Opcode::from_byte(self.peek_byte()));
            println!("---");
            self.step()?;
        }

        // return current state
        mem::drop(mem::replace(&mut self.chunk, old_chunk));
        self.ip = old_ip;

        // nothing went wrong!
        return Result::Ok(());
    }

    // TODO: there are a lot of optimizations that can be made
    // I'll list a few here:
    // - searching the stack for variables
    //   A global Hash-table has significantly less overhead for function calls
    // - cloning the heck out of everything - useless copies
    //   instead, lifetime analysis during compilation
    // - replace some panics with Result<()>s

    /// Load a constant and push it onto the stack.
    fn con(&mut self) -> Result<(), Trace> {
        // get the constant index
        let index = self.next_number();

        self.stack.push_data(self.chunk.constants[index].clone());
        self.done()
    }

    /// Save the topmost value on the stack into a variable.
    fn save(&mut self) -> Result<(), Trace> {
        let data = self.stack.pop_data();
        let index = self.next_number();
        self.stack.set_local(index);
        self.done()
    }

    /// Push a copy of a variable's value onto the stack.
    fn load(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        self.stack.get_local(index);
        self.done()
    }

    /// Delete the top item of the stack
    fn del(&mut self) -> Result<(), Trace> {
        Ok(mem::drop(self.stack.pop_data()))
    }

    // TODO: closures
    /// Call a function on the top of the stack, passing the next value as an argument.
    fn call(&mut self) -> Result<(), Trace> {
        let fun = match self.stack.pop_data() {
            Data::Lambda(l) => l,
            _               => unreachable!(),
        };
        let arg = self.stack.pop_data();

        self.stack.push_frame();
        self.stack.push_data(arg);
        println!("entering...");
        // TODO: keep the passerine call stack separated from the rust call stack.
        self.run(fun)?;
        println!("exiting...");

        self.done()
    }

    /// Return a value from a function.
    /// End the execution of the current chunk.
    /// Relpaces the last frame with the value on the top of the stack.
    /// Expects the stack to be a `[..., Frame, Data]`
    fn return_val(&mut self) -> Result<(), Trace> {
        let val = self.stack.pop_data();
        self.stack.pop_frame();
        self.stack.push_data(val);
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
    use crate::common::source::Source;

    #[test]
    fn init_run() {
        // TODO: check @ each step, write more tests
        let chunk = gen(parse(lex(
            Source::source("boop = 37.201; true; dhuiew = true; boop")
        ).unwrap()).unwrap());

        let mut vm = VM::init();

        match vm.run(chunk) {
            Ok(_)  => (),
            Err(e) => eprintln!("{}", e),
        }
    }

    #[test]
    fn block_expression() {
        let chunk = gen(parse(lex(
            Source::source("boop = true; heck = { x = boop; x }; heck")
        ).unwrap()).unwrap());

        let mut vm = VM::init();

        match vm.run(chunk) {
            Ok(_)  => (),
            Err(e) => eprintln!("{}", e),
        }

        match vm.stack.pop_data() {
            Data::Boolean(true) => (),
            _                   => panic!("Expected true value"),
        }
    }

    #[test]
    fn functions() {
        let chunk = gen(parse(lex(
            Source::source("iden = x -> x; y = true; iden ((iden iden) (iden y))")
        ).unwrap()).unwrap());

        let mut vm = VM::init();
        match vm.run(chunk) {
            Ok(_)  => (),
            Err(e) => eprintln!("{}", e),
        }

        let t = vm.stack.pop_data();
        assert_eq!(t, Data::Boolean(true));
    }

    #[test]
    fn fun_scope() {
        let chunk = gen(parse(lex(
            Source::source("y = (x -> { z = x; z }) 7.0; y")
        ).unwrap().into()).unwrap());

        println!("{:#?}", chunk);

        let mut vm = VM::init();
        match vm.run(chunk) {
            Ok(_)  => (),
            Err(e) => eprintln!("{}", e),
        }

        // check that y is in fact 7
        let t = vm.stack.pop_data();
        assert_eq!(t, Data::Real(7.0));
    }
}
