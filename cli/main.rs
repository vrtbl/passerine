use p;
use p::utils;
use p::vm;

fn main() {
    let source = /* get source file */;

    // pipe source through compiler to generate bytecode
    let bytecode = match util::pipes::all(source) {
        Ok(b)  => b,
        Err(e) => utils::error::display(e);
    };

    let mut vm = vm::VM::init();   // initialize vm
    // utils::pipes::std(&mut vm);    // import passerine standard library
    let result = vm.run(bytecode); // run bytecode

    // if there was an error, display it.
    match result {
        Ok(()) => (),
        Err(e) => utils::error::display(e),
    }
}
