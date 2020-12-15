use std::mem;

use crate::common::{
    number::build_number,
    data::Data,
    opcode::Opcode,
    lambda::{Captured, Lambda},
    closure::Closure,
    span::Span,
};

use crate::vm::{
    trace::Trace,
    stack::Stack,
};

/// A `VM` executes bytecode lambda closures.
/// (That's a mouthful - think bytecode + some context).
/// VM initialization overhead is tiny,
/// and each VM's state is self-contained,
/// so more than one can be spawned if needed.
#[derive(Debug)]
pub struct VM {
    pub closure: Closure,
    pub stack:   Stack,
    pub ip:      usize,
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
    #[inline]
    pub fn next(&mut self)                           { self.ip += 1; }
    /// Jumps past the end of the block, causing the current lambda to return.
    #[inline]
    pub fn terminate(&mut self) -> Result<(), Trace> { self.ip = self.closure.lambda.code.len(); Ok(()) }
    /// Advances IP, returns `Ok`. Used in Bytecode implementations.
    #[inline]
    pub fn done(&mut self)      -> Result<(), Trace> { self.next(); Ok(()) }
    /// Returns the current instruction as a byte.
    #[inline]
    pub fn peek_byte(&mut self) -> u8                { self.closure.lambda.code[self.ip] }
    /// Advances IP and returns the current instruction as a byte.
    #[inline]
    pub fn next_byte(&mut self) -> u8                { self.next(); self.peek_byte() }

    /// Builds the next number in the bytecode stream.
    /// See `utils::number` for more.
    #[inline]
    pub fn next_number(&mut self) -> usize {
        self.next();
        let remaining      = &self.closure.lambda.code[self.ip..];
        let (index, eaten) = build_number(remaining);
        self.ip += eaten - 1; // ip left on next op
        return index;
    }

    #[inline]
    pub fn current_span(&self) -> Span {
        self.closure.lambda.index_span(self.ip)
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
            Opcode::Copy    => self.copy_val(),
            Opcode::Capture => self.capture(),
            Opcode::Save    => self.save(),
            Opcode::SaveCap => self.save_cap(),
            Opcode::Load    => self.load(),
            Opcode::LoadCap => self.load_cap(),
            Opcode::Call    => self.call(),
            Opcode::Return  => self.return_val(),
            Opcode::Closure => self.closure(),
            Opcode::Print   => self.print(),
            Opcode::Label   => self.label(),
            Opcode::UnLabel => self.un_label(),
            Opcode::UnData  => self.un_data(),
            Opcode::FFICall => self.ffi_call(),
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
                // println!("Error!");
                break;
            };
            // println!("---");
        }
        // println!("after: {:?}", self.stack.stack);
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
    #[inline]
    pub fn con(&mut self) -> Result<(), Trace> {
        // get the constant index
        let index = self.next_number();

        self.stack.push_data(self.closure.lambda.constants[index].clone());
        self.done()
    }

    /// Moves the top value on the stack to the heap,
    /// replacing it with a reference to the heapified value.
    #[inline]
    pub fn capture(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        self.stack.heapify(index);   // move value to the heap
        self.done()
    }

    /// Save the topmost value on the stack into a variable.
    #[inline]
    pub fn save(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        self.stack.set_local(index);
        self.done()
    }

    /// Save the topmost value on the stack into a captured variable.
    #[inline]
    pub fn save_cap(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        let data  = self.stack.pop_data();
        mem::drop(self.closure.captures[index].replace(data));
        self.done()
    }

    /// Push a copy of a variable's value onto the stack.
    #[inline]
    pub fn load(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        let data = self.stack.local_data(index);
        self.stack.push_data(data);
        self.done()
    }

    /// Load a captured variable from the current closure.
    #[inline]
    pub fn load_cap(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        // NOTE: should heaped data should only be present for variables?
        // self.closure.captures[index].borrow().to_owned()
        self.stack.push_data(self.closure.captures[index].borrow().to_owned());
        self.done()
    }

    /// Delete the top item of the stack.
    #[inline]
    pub fn del(&mut self) -> Result<(), Trace> {
        mem::drop(self.stack.pop_data());
        self.done()
    }

    /// Copy the top data of the stack, i.e.
    /// `[F, D]` becomes `[F, D, D]`.
    #[inline]
    pub fn copy_val(&mut self) -> Result<(), Trace> {
        let data = self.stack.pop_data();
        self.stack.push_data(data.clone());
        self.stack.push_data(data);
        self.done()
    }

    #[inline]
    pub fn print(&mut self) -> Result<(), Trace> {
        let data = self.stack.pop_data();
        println!("{}", data);
        self.stack.push_data(data);
        self.done()
    }

    #[inline]
    pub fn label(&mut self) -> Result<(), Trace> {
        let kind = match self.stack.pop_data() {
            Data::Kind(n) => n,
            _ => unreachable!(),
        };
        let data = self.stack.pop_data();
        self.stack.push_data(Data::Label(Box::new(kind), Box::new(data)));
        self.done()
    }

    fn un_label(&mut self) -> Result<(), Trace> {
        let kind = match self.stack.pop_data() {
            Data::Kind(n) => n,
            _ => unreachable!(),
        };

        let d = match self.stack.pop_data() {
            Data::Label(n, d) if *n == kind => d,
            other => return Err(Trace::error(
                "Pattern Matching",
                &format!("The data '{}' does not match the Label '{}'", other, kind),
                vec![self.current_span()],
            )),
        };

        self.stack.push_data(*d);
        self.done()
    }

    fn un_data(&mut self) -> Result<(), Trace> {
        let expected = self.stack.pop_data();
        let data = self.stack.pop_data();

        if data != expected {
            return Err(Trace::error(
                "Pattern Matching",
                &format!("The data '{}' does not match the expected data '{}'", data, expected),
                vec![self.current_span()],
            ));
        }

        self.done()
    }

    /// Call a function on the top of the stack, passing the next value as an argument.
    pub fn call(&mut self) -> Result<(), Trace> {
        let fun = match self.stack.pop_data() {
            Data::Closure(c) => *c,
            o => return Err(Trace::error(
                "Call",
                &format!("The data '{}' is not a function and can not be called", o),
                vec![self.current_span()],
            )),
        };
        let arg = self.stack.pop_data();

        self.stack.push_frame();
        self.stack.push_data(arg);
        // println!("entering...");
        // TODO: keep the passerine call stack separated from the rust call stack.
        match self.run(fun) {
            Ok(()) => (),
            Err(mut trace) => {
                trace.add_context(self.current_span());
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

    pub fn closure(&mut self) -> Result<(), Trace> {
        let index = self.next_number();

        let lambda = match self.closure.lambda.constants[index].clone() {
            Data::Lambda(lambda) => *lambda,
            _ => unreachable!("Expected a lambda to be wrapped with a closure"),
        };

        let mut closure = Closure::wrap(lambda);

        for captured in closure.lambda.captures.iter() /* .rev */ {
            let reference = match captured {
                Captured::Local(index) => {
                    match self.stack.local_data(*index) {
                        Data::Heaped(h) => h,
                        _ => unreachable!("Expected data to be on the heap"),
                    }
                },
                Captured::Nonlocal(upvalue) => self.closure.captures[*upvalue].clone(),
            };
            closure.captures.push(reference)
        }

        self.stack.push_data(Data::Closure(Box::new(closure)));
        self.done()
    }

    pub fn ffi_call(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        let ffi_function = &self.closure.lambda.ffi[index];

        let argument = self.stack.pop_data();
        let returned = match (ffi_function.0)(argument) {
            Ok(d) => d,
            Err(e) => return Err(Trace::error(
                "FFI Call", &e, vec![self.current_span()],
            )),
        };

        self.stack.push_data(returned);
        self.done()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::{
        parse::parse,
        desugar::desugar,
        lex::lex,
        gen::gen,
    };
    use crate::common::source::Source;

    fn inspect(source: &str) -> VM {
        let lambda = lex(Source::source(source))
            .and_then(parse)
            .and_then(desugar)
            .and_then(gen)
            .map_err(|e| println!("{}", e))
            .unwrap();

        // println!("{:?}", lambda);
        let mut vm = VM::init();

        match vm.run(Closure::wrap(lambda)) {
            Ok(()) => vm,
            Err(e) => {
                println!("{}", e);
                panic!();
            },
        }
    }

    #[test]
    fn init_run() {
        inspect("x = 0.0");
    }

    #[test]
    fn block_expression() {
        inspect("x = false; boop = true; heck = { x = boop; x }; heck");
    }

    #[test]
    fn functions() {
        let mut vm = inspect("iden = x -> x; y = true; iden ({ y = false; iden iden } (iden y))");
        let identity = vm.stack.pop_data();
        assert_eq!(identity, Data::Boolean(true));
    }

    #[test]
    fn fun_scope() {
        // y = (x -> { y = x; y ) 7.0; y
        let mut vm = inspect("one = 1.0\npi = 3.14\ne = 2.72\n\nx = w -> pi\nx 37.6");
        let pi = vm.stack.pop_data();
        assert_eq!(pi, Data::Real(3.14));
    }

    #[test]
    fn mutate_capture() {
        inspect("odd = (); even = x -> odd; odd = 1.0; even (); odd");
    }

    #[test]
    fn mutate_capture_fn() {
        inspect("\
            pi = 3.14\n\
            printpi = x -> print pi\n\
            \n\
            redef = ()\n\
            redef = w -> {\n    \
                w (printpi ())\n\
            }\n\
            \n\
            redef printpi\n\
        ");
    }

    // TODO: figure out how to make the following passerine code into a test
    // without entering into an infinite loop (which is the intended behaviour)
    // loop = ()
    // loop = y -> x -> {
    //     print y
    //     print x
    //     loop x y
    // }
    //
    // loop true false
}
