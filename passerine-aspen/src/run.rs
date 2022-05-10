use std::path::PathBuf;

// use passerine::{
//     common::{
//         closure::Closure,
//         source::Source,
//     },
//     compiler::{
//         desugar,
//         gen,
//         hoist,
//         lex,
//         parse,
//     },
//     vm::VM,
// };
use crate::{
    manifest::Manifest,
    ENTRYPOINT,
    SOURCE,
};

pub fn run(path: PathBuf) -> Result<(), String> {
    // just one file, for now
    todo!()
    // let (_manifest, path) = Manifest::package(&path)?;
    // let file = path.join(SOURCE).join(ENTRYPOINT);

    // let source = Source::path(&file).map_err(|_| {
    //     format!(
    //         "Could not find source entrypoint '{}/{}'",
    //         SOURCE, ENTRYPOINT
    //     )
    // })?;

    // let tokens = lex(source).map_err(|e| e.to_string())?;
    // let ast = parse(tokens).map_err(|e| e.to_string())?;
    // let cst = desugar(ast).map_err(|e| e.to_string())?;
    // let sst = hoist(cst).map_err(|e| e.to_string())?;
    // let bytecode = gen(sst).map_err(|e| e.to_string())?;

    // let mut vm = VM::init(Closure::wrap(bytecode));
    // vm.run().map_err(|e| e.to_string())?;

    // Ok(())
}
