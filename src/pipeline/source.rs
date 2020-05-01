use std::path::Path;
use std::fs::File;

// represents some literal source code
// TODO: can be iterated over by graphemes

pub struct Source {
    pub contents: String,
    pub path:     Path,
}

impl Source {
    pub fn path(path: Path) -> Option<Source> {
        let mut file   = File::open(path)?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;

        Some(Source { source, path })
    }

    pub fn source(source: String) -> Source {
        Source { source, path: Path::new("./source") }
    }

    pub fn new(source: String, path: Path) -> Source {
        Source { source, path }
    }
}
