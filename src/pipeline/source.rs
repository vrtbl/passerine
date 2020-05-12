use std::str::Chars;
use std::path::PathBuf;
use std::io::Read;
use std::fs::File;

/// `Source` represents some literal source code.
/// Whether a repl session, a file on disk, or some library code.
/// It's essentially a string with a path, the path serving as the source's name.
/// Source files without a path point to `./source`,
/// though this behaviour might change in the future.
#[derive(Debug, PartialEq, Eq)]
pub struct Source {
    pub contents: String,
    pub path:     PathBuf,
}

impl Source {
    /// Creates a new `Source` given both an `&str` and a `PathBuf`.
    /// Note that this function does not check that the contents of the file
    /// match the source.
    /// `Source::path` or `Source::source` should be used instead.
    pub fn new(source: &str, path: PathBuf) -> Source {
        Source { contents: source.to_string(), path }
    }

    /// Build a `Source` from a path.
    /// This will read a file to create a new source.
    pub fn path(path: PathBuf) -> std::io::Result<Source> {
        let mut source = String::new();
        let mut file   = File::open(path.clone())?;
        file.read_to_string(&mut source)?;

        Ok(Source { contents: source, path })
    }

    /// Build an empty `Source` containing just a string.
    /// Note that this source will point towards `./source`.
    pub fn source(source: &str) -> Source {
        Source { contents: source.to_string(), path: PathBuf::from("./source") }
    }
}
