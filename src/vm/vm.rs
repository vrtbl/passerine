use std::mem;

use crate::common::{
    number::build_number,
    data::Data,
    opcode::Opcode,
    lambda::Lambda,
    span::Span,
};

use crate::vm::{
    trace::Trace,
    // tag::Tagged,
    stack::Stack,
    closure::Closure,
};

/// A `VM` executes bytecode lambda closures.
/// (That's a mouthful - think bytecode + some context).
/// VM initialization overhead is tiny,
/// and each VM's state is self-contained,
/// so more than one can be spawned if needed.
#[derive(Debug)]
pub struct VM {
    closure: Closure,
    stack:   Stack,
    ip:      usize,
}

// NOTE: use Opcode::same and Opcode.to_byte() rather than actual bytes
// Don't worry, the compiler *should* get rid of this overhead and just use bytes

// this impl contains initialization, helper functions, and the core interpreter loop
// the next impl contains opcode implementations
impl VM {
    /// Initialize a new VM.
    /// To run the VM, a lambda must be passed to it through `run`.
    pub fn init() -> VM {
        VM {
            closure: Closure::wrap(Lambda::empty()),
            stack: Stack::init(),
            ip:    0,
        }
    }

    /// Advances to the next instruction.
    pub fn next(&mut self)                           { self.ip += 1; }
    /// Jumps past the end of the block, causing the current lambda to return.
    pub fn terminate(&mut self) -> Result<(), Trace> { self.ip = self.closure.lambda.code.len(); Ok(()) }
    /// Advances IP, returns `Ok`. Used in Bytecode implementations.
    pub fn done(&mut self)      -> Result<(), Trace> { self.next(); Ok(()) }
    /// Returns the current instruction as a byte.
    pub fn peek_byte(&mut self) -> u8                { self.closure.lambda.code[self.ip] }
    // Advances IP and returns the current instruction as a byte.
    pub fn next_byte(&mut self) -> u8                { self.next(); self.peek_byte() }

    /// Builds the next number in the bytecode stream.
    /// See `utils::number` for more.
    pub fn next_number(&mut self) -> usize {
        self.next();
        let remaining      = &self.closure.lambda.code[self.ip..];
        let (index, eaten) = build_number(remaining);
        self.ip += eaten - 1; // ip left on next op
        return index;
    }

    // core interpreter loop

    /// Dissasembles and interprets a single (potentially fallible) bytecode op.
    /// The op definitions follow in the next `impl` block.
    /// To see what each op does, check `common::opcode::Opcode`.
    pub fn step(&mut self) -> Result<(), Trace> {
        let opcode = Opcode::from_byte(self.peek_byte());

        match opcode {
            Opcode::Con     => self.con(),
            Opcode::Del     => self.del(),
            Opcode::Capture => self.capture(),
            Opcode::Save    => self.save(),
            Opcode::SaveCap => self.save_cap(),
            Opcode::Load    => self.load(),
            Opcode::LoadCap => self.load_cap(),
            Opcode::Call    => self.call(),
            Opcode::Return  => self.return_val(),
        }
    }

    /// Suspends the current lambda and runs a new one on the VM.
    /// Runs until either success, in which it restores the state of the previous lambda,
    /// Or failure, in which it returns the runtime error.
    /// In the future, fibers will allow for error handling -
    /// right now, error in Passerine are practically panics.
    pub fn run(&mut self, closure: Closure) -> Result<(), Trace> {
        // cache current state, load new bytecode
        let old_closure = mem::replace(&mut self.closure, closure);
        let old_ip      = mem::replace(&mut self.ip,    0);
        // TODO: should lambdas store their own ip?

        let mut result = Ok(());

        while self.ip < self.closure.lambda.code.len() {
            // println!("before: {:?}", self.stack.stack);
            // println!("executing: {:?}", Opcode::from_byte(self.peek_byte()));
            if let error @ Err(_) = self.step() {
                // TODO: clean up stack on error
                result = error;
                break;
            };
            // println!("---");
        }
        // println!("after: {:?}", self.stack);
        // println!("---");

        // return current state
        mem::drop(mem::replace(&mut self.closure, old_closure));
        self.ip = old_ip;

        // If something went wrong, the error will be returned.
        return result;
    }

    // TODO: there are a lot of optimizations that can be made
    // I'll list a few here:
    // - searching the stack for variables
    //   A global Hash-table has significantly less overhead for function calls
    // - cloning the heck out of everything - useless copies
    //   instead, lifetime analysis during compilation
    // - replace some panics with Result<()>s

    /// Load a constant and push it onto the stack.
    pub fn con(&mut self) -> Result<(), Trace> {
        // get the constant index
        let index = self.next_number();

        self.stack.push_data(self.closure.lambda.constants[index].clone());
        self.done()
    }

    /// Moves the top value on the stack to the heap,
    /// replacing it with a reference to the heapified value.
    /// > NOTE: Behaviour is not stabilized yet.
    pub fn capture(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        // TODO: Write this all out efficiently?
        self.stack.heapify(index);
        self.stack.get_local(index);
        if let Data::Heaped(captured) = self.stack.pop_data() {
            self.closure.captured.push(captured);
        } else {
            unreachable!("Heaped data wasn't heaped when cloned to top")
        }
        self.done()
    }

    /// Save the topmost value on the stack into a variable.
    pub fn save(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        self.stack.set_local(index);
        self.done()
    }

    /// Save the topmost value on the stack into a captured variable.
    /// > NOTE: Not implemented.
    pub fn save_cap(&mut self) -> Result<(), Trace> {
        Err(Trace::error(
            "Impl.",
            "Mutating captured variables is not yet implemented",
            vec![Span::empty()]
        ))
        // unimplemented!();
    }

    /// Push a copy of a variable's value onto the stack.
    pub fn load(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        self.stack.get_local(index);
        self.done()
    }

    /// Load a captured variable from the current closure.
    /// TODO: simplify mechanisms
    pub fn load_cap(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        self.stack.push_data(Data::Heaped(self.closure.captured[index].clone()));

        // Err(Trace::error(
        //     "Impl.",
        //     "Referencing captured variables is not yet implemented",
        //     vec![self.closure.lambda.index_span(self.ip)],
        // ));
        // unimplemented!();

        self.done()
    }

    /// Delete the top item of the stack.
    pub fn del(&mut self) -> Result<(), Trace> {
        mem::drop(self.stack.pop_data());
        self.done()
    }

    // TODO: closures
    /// Call a function on the top of the stack, passing the next value as an argument.
    pub fn call(&mut self) -> Result<(), Trace> {
        let fun = match self.stack.pop_data() {
            Data::Lambda(l) => Closure::wrap(l),
            _               => unreachable!(),
        };
        let arg = self.stack.pop_data();

        self.stack.push_frame();
        self.stack.push_data(arg);
        // println!("entering...");
        // TODO: keep the passerine call stack separated from the rust call stack.
        match self.run(fun) {
            Ok(()) => (),
            Err(mut trace) => {
                trace.add_context(self.closure.lambda.index_span(self.ip));
                return Err(trace);
            },
        };
        // println!("exiting...");

        self.done()
    }

    /// Return a value from a function.
    /// End the execution of the current lambda.
    /// Takes the number of locals on the stack
    /// Relpaces the last frame with the value on the top of the stack.
    /// Expects the stack to be a `[..., Frame, Local 1, ..., Local N, Data]`
    pub fn return_val(&mut self) -> Result<(), Trace> {
        // the value to be returned
        let val = self.stack.pop_data();

        // clear all locals
        let locals = self.next_number();
        for _ in 0..locals { self.del()?; }

        self.stack.pop_frame();    // remove the frame
        self.stack.push_data(val); // push the return value
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

        let lambda = gen(parse(lex(
            Source::source("heck = 0.0; y = heck")
        ).unwrap()).unwrap()).unwrap();

        // println!("{:?}", lambda);
        let mut vm = VM::init();

        match vm.run(Closure::wrap(lambda)) {
            Ok(_)  => (),
            Err(e) => eprintln!("{}", e),
        }
    }

    #[test]
    fn block_expression() {
        let lambda = gen(parse(lex(
            Source::source("x = false; boop = true; heck = { x = boop; x }; heck")
        ).unwrap()).unwrap()).unwrap();

        // println!("{:?}", lambda);
        let mut vm = VM::init();

        match vm.run(Closure::wrap(lambda)) {
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
        let lambda = gen(parse(lex(
            Source::source("iden = x -> x; y = true; iden ({y = false; iden iden} (iden y))")
        ).unwrap()).unwrap()).unwrap();

        println!("{:?}", lambda);
        let mut vm = VM::init();

        match vm.run(Closure::wrap(lambda)) {
            Ok(_)  => (),
            Err(e) => eprintln!("{}", e),
        }

        let t = vm.stack.pop_data();
        assert_eq!(t, Data::Boolean(true));
    }

    #[test]
    fn fun_scope() {
        // y = (x -> { y = x; y ) 7.0; y
        let lambda = gen(
            parse(
                lex(Source::source("pi = 3.15; x = w -> pi; x ()")).unwrap()
            ).unwrap()
        ).unwrap();

        let mut vm = VM::init();

        match vm.run(Closure::wrap(lambda)) {
            Ok(_)  => (),
            Err(e) => eprintln!("{}", e),
        }

        println!("{:?}", vm.stack.stack);

        panic!();

        // // check that y is in fact 7
        // let t = vm.stack.pop_data();
        // assert_eq!(t, Data::Real(7.0));
    }
}
