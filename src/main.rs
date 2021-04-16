use passerine::{
    common::{closure::Closure, source::Source},
    compiler::{lex, parse, desugar, gen, hoist},
    vm::vm::VM,
};

pub fn main() -> Result<(), String> {
    let path = std::env::args_os().nth(1).expect("Usage: <path>");
    
    let source = Source::path(path.as_ref())
        .map_err(|_| format!("Could not find source entrypoint {:?}", path))?;

    let tokens    =   lex(source).map_err(|e| e.to_string())?;
    let ast       = parse(tokens).map_err(|e| e.to_string())?;
    let cst       =  desugar(ast).map_err(|e| e.to_string())?;
    let sst       =    hoist(cst).map_err(|e| e.to_string())?;
    let bytecode  =      gen(sst).map_err(|e| e.to_string())?;

    let mut vm = VM::init(Closure::wrap(bytecode));
    vm.run().map_err(|e| e.to_string())?;

    Ok(())
}