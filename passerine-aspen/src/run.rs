use std::path::PathBuf;

use passerine::{
    compile,
    Source,
};

use crate::{
    manifest::Manifest,
    ENTRYPOINT,
    SOURCE,
};

pub fn run(path: PathBuf) -> Result<(), String> {
    // just one file, for now
    let (_manifest, path) = Manifest::package(&path)?;
    let file = path.join(SOURCE).join(ENTRYPOINT);

    let source = Source::path(&file).map_err(|_| {
        format!(
            "Could not find source entrypoint '{}/{}'",
            SOURCE, ENTRYPOINT
        )
    })?;

    let bytecode = compile(source).map_err(|e| e.to_string())?;

    println!("{:#?}", bytecode.lambda);
    println!("{}", bytecode.lambda);

    // let mut vm = VM::init(Closure::wrap(bytecode));
    // vm.run().map_err(|e| e.to_string())?;

    Ok(())
}
