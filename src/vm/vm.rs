use std::mem;

use crate::common::{
    number::build_number,
    data::Data,
    opcode::Opcode,
    lambda::Captured,
    closure::Closure,
    span::Span,
};

use crate::vm::{
    trace::Trace,
    slot::Suspend,
    stack::Stack,
};

// TODO: algebraic effects
// more than just Trace, Runtime - mechanism for raising effects
// fiber scheduling environment handles FFI, no more holding refs to rust functions.
// TODO: convert VM to Fiber

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
    pub fn init(closure: Closure) -> VM {
        let mut vm = VM {
            closure,
            stack: Stack::init(),
            ip:    0,
        };
        vm.stack.declare(vm.closure.lambda.decls);
        return vm;
    }

    /// Advances to the next instruction.
    #[inline]
    pub fn next(&mut self)                           { self.ip += 1; }
    /// Advances IP, returns `Ok`. Used in Bytecode implementations.
    #[inline]
    pub fn done(&mut self)      -> Result<(), Trace> { self.next(); Ok(()) }
    /// Returns the current instruction as a byte.
    #[inline]
    pub fn peek_byte(&mut self) -> u8                { self.closure.lambda.code[self.ip] }
    /// Advances IP and returns the current instruction as a byte.
    #[inline]
    pub fn next_byte(&mut self) -> u8                { self.next(); self.peek_byte() }

    /// Returns whether the program has terminated
    #[inline]
    pub fn is_terminated(&mut self) -> bool {
        self.ip >= self.closure.lambda.code.len()
    }

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
            Opcode::NotInit => self.not_init(),
            Opcode::Del     => self.del(),
            Opcode::FFICall => self.ffi_call(),
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
            Opcode::Tuple   => self.tuple(),
            Opcode::UnData  => self.un_data(),
            Opcode::UnLabel => self.un_label(),
            Opcode::UnTuple => self.un_tuple(),
            Opcode::Noop    => self.done(),
        }
    }

    pub fn unwind(&mut self) {
        // restore suspended callee
        let suspend = self.stack.pop_frame(); // remove the frame
        self.ip      = suspend.ip;
        self.closure = suspend.closure;

        // indicate failure
        self.stack.push_not_init();
    }

    /// Suspends the current lambda and runs a new one on the VM.
    /// Runs until either success, in which it restores the state of the previous lambda,
    /// Or failure, in which it returns the runtime error.
    /// In the future, fibers will allow for error handling -
    /// right now, error in Passerine are practically panics.
    pub fn run(&mut self) -> Result<(), Trace> {
        // println!("Starting\n{}", self.closure.lambda);
        let mut result = Ok(());

        while !self.is_terminated() {
            // println!("before: {:#?}", self.stack.stack);
            // println!("executing: {:?}", Opcode::from_byte(self.peek_byte()));
            result = self.step();
            if result.is_err() { break; }
            // println!("---");
        }
        // println!("after: {:?}", self.stack.stack);
        // println!("---");

        if let Err(mut trace) = result {
            while self.stack.unwind_frame() {
                self.unwind();
                self.ip -= 1;
                trace.add_context(self.current_span());
            }

            result = Err(trace);
        };

        return result;
    }

    /// Load a constant and push it onto the stack.
    #[inline]
    pub fn con(&mut self) -> Result<(), Trace> {
        // get the constant index
        let index = self.next_number();

        self.stack.push_data(self.closure.lambda.constants[index].clone());
        self.done()
    }

    #[inline]
    pub fn not_init(&mut self) -> Result<(), Trace> {
        self.stack.push_not_init();
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
        let mut data = self.stack.local_data(index);

        if let Data::Heaped(d) = data { data = d.borrow().to_owned() };
        if let Data::NotInit = data {
            return Err(Trace::error(
                "Reference",
                &format!("This local variable was referenced before assignment"),
                vec![self.current_span()],
            ));
        };

        self.stack.push_data(data);
        self.done()
    }

    /// Load a captured variable from the current closure.
    #[inline]
    pub fn load_cap(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        let data = self.closure.captures[index].borrow().to_owned();

        if let Data::NotInit = data {
            return Err(Trace::error(
                "Reference",
                &format!("This captured variable was referenced before assignment"),
                vec![self.current_span()],
            ));
        };

        self.stack.push_data(data);
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
        self.stack.push_data(Data::Label(kind, Box::new(data)));
        self.done()
    }

    #[inline]
    pub fn tuple(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        let mut items = vec![];
        for _ in 0..index {
            items.push(self.stack.pop_data())
        }

        items.reverse();
        self.stack.push_data(Data::Tuple(items));
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

    fn un_label(&mut self) -> Result<(), Trace> {
        let kind = match self.stack.pop_data() {
            Data::Kind(n) => n,
            _ => unreachable!(),
        };

        let d = match self.stack.pop_data() {
            Data::Label(n, d) if n == kind => d,
            other => return Err(Trace::error(
                "Pattern Matching",
                &format!("The data '{}' does not match the Label '{}'", other, kind),
                vec![self.current_span()],
            )),
        };

        self.stack.push_data(*d);
        self.done()
    }

    fn un_tuple(&mut self) -> Result<(), Trace> {
        let index = self.next_number();
        let t = match self.stack.pop_data() {
            Data::Tuple(t) => t,
            other => return Err(Trace::error(
                "Pattern Matching",
                &format!("The data '{}' is not a tuple", other),
                vec![self.current_span()],
            )),
        };

        let length = t.len();
        if index >= length {
            return Err(Trace::error(
                "Indexing",
                &format!(
                    "The tuple '{}' is of length {}, so the index {} is out-of-bounds",
                    Data::Tuple(t), length, index
                ),
                vec![self.current_span()],
            ));
        }

        let data = t[index].clone();
        self.stack.push_data(Data::Tuple(t));
        self.stack.push_data(data);
        self.done()
    }

    /// Call a function on the top of the stack, passing the next value as an argument.
    pub fn call(&mut self) -> Result<(), Trace> {
        // get the function and argument to run
        let fun = match self.stack.pop_data() {
            Data::Closure(c) => *c,
            o => return Err(Trace::error(
                "Call",
                &format!("The data '{}' is not a function and can not be called", o),
                vec![self.current_span()],
            )),
        };
        let arg = self.stack.pop_data();

        // TODO: make all programs end in return,
        // so bounds check (i.e. is_terminated) is never required
        self.next();
        let tail_call = !self.is_terminated()
                     && Opcode::Return
                     == Opcode::from_byte(self.peek_byte());

        // clear the stack if there's a tail call
        // we must do this before we suspend the calling context
        if tail_call {
            let locals = self.next_number();
            for _ in 0..locals { self.del()?; }
        }

        // suspend the calling context
        let old_closure = mem::replace(&mut self.closure, fun);
        let old_ip      = mem::replace(&mut self.ip,      0);
        let suspend = Suspend {
            ip: old_ip,
            closure: old_closure,
        };

        // if there's a tail call, we don't bother pushing a new frame
        // the topmost frame doesn't carry any context;
        // that context is intrinsic to the VM itself.
        if !tail_call {
            self.stack.push_frame(suspend);
        }

        // set up the stack for the function call
        // self.stack.push_frame(suspend);
        self.stack.declare(self.closure.lambda.decls);
        self.stack.push_data(arg);

        // println!("{}", self.closure.lambda);

        Ok(())
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

        // restore suspended callee
        let suspend = self.stack.pop_frame(); // remove the frame
        self.ip      = suspend.ip;
        self.closure = suspend.closure;

        // push return value
        self.stack.push_data(val); // push the return value
        Ok(())
    }

    pub fn closure(&mut self) -> Result<(), Trace> {
        let index = self.next_number();

        let lambda = match self.closure.lambda.constants[index].clone() {
            Data::Lambda(lambda) => lambda,
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
        let returned = match ffi_function.call(argument) {
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
    use crate::compile;
    use crate::common::source::Source;

    fn inspect(source: &str) -> VM {
        let closure = compile(Source::source(source))
            .map_err(|e| println!("{}", e))
            .unwrap();

        let mut vm = VM::init(closure);

        match vm.run() {
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
            printpi = x -> println pi\n\
            \n\
            redef = ()\n\
            redef = w -> {\n    \
                w (printpi ())\n\
            }\n\
            \n\
            redef printpi\n\
        ");
    }

    #[test]
    fn hoist_later() {
        inspect("\
            w = 0.5
            later = n -> thing 10.0 - w\n\
            thing = x -> x + 20.0\n\
            -- later 5.0\n\
        ");
    }

    // TODO: figure out how to make the following passerine code into a test
    // without entering into an infinite loop (which is the intended behaviour)
    // maybe try running it a large number of times,
    // and check the size of the stack?
    // loop = ()
    // loop = y -> x -> {
    //     print y
    //     print x
    //     loop x y
    // }
    //
    // loop true false
}
