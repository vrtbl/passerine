use std::{env::current_dir, ffi::OsStr, path::PathBuf};

use structopt::StructOpt;

pub fn package_dir(path: &OsStr) -> PathBuf {
    return if path == "." {
        current_dir().expect("Can not determine package directory")
    } else {
        PathBuf::from(path)
    };
}

#[derive(StructOpt, Debug)]
pub struct Package {
    #[structopt(default_value = ".", parse(from_os_str = package_dir))]
    pub path: PathBuf,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Aspen", bin_name = "aspen", about)]
pub enum Aspen {
    /// Creates a new Passerine package
    New(Package),
    // Update,
    // Publish,
    /// Runs the specified package
    Run(Package),
    Repl,
    // Test,
    // Bench,
    // Doc,
    // Debug,
}
