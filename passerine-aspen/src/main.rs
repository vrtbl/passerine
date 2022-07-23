use structopt::StructOpt;

// argument parser and configuation
pub mod cli;
pub mod manifest;
pub mod status;

// command implementations
pub mod add;
pub mod bench;
pub mod debug;
pub mod doc;
pub mod new;
pub mod publish;
pub mod repl;
pub mod run;
pub mod test;
pub mod update;

use crate::{cli::Aspen, status::Status};

// TODO: handle this passerine side
pub const MANIFEST: &str = "aspen.toml";
pub const SOURCE: &str = "src";
pub const ENTRYPOINT: &str = "main.pn";

fn main() {
    let subcommand = Aspen::from_args();

    let result = match subcommand {
        Aspen::New(package) => new::new(package.path),
        Aspen::Run(package) => run::run(package.path),
        Aspen::Repl => repl::repl(),
        _ => unimplemented!(),
    };

    if let Err(r) = result {
        Status::fatal().log(&r)
    }
}
