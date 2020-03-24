use crate::pipeline::error::Error;
use crate::pipeline::bytecode::Bytecode;
use crate::vm::vm::VM;
use crate::pipes::*;

pub fn all(source: &str) -> Result<Bytecode, Error> {
    // TODO: handle errors
    unimplemented!();

    let tokens   = lex::lex(source)?;
    let ast      = parse::parse(tokens)?;
    let bytecode = gen::gen(ast)?;

    return bytecode;
}

// how long does it take to build the stdlib?
#[bench]
pub fn stdlib(vm: &mut VM) {
    // TODO: cache vm or bytecode?

    // 'compile' standard library
    let source = /* TODO: path to standard library */ unimplemented!();
    let bytecode = all(source);

    // run the bytecode on the vm
    vm.run(bytecode);
}
