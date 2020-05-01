use std::path::PathBuf;
use std::io::Read;
use std::fs::File;

// represents some literal source code
pub struct Source {
    pub contents: String,
    pub path:     PathBuf,
}

impl Source {
    pub fn new(source: &str, path: PathBuf) -> Source {
        assert!(path.is_file());
        Source { contents: source.to_string(), path }
    }

    pub fn path(path: PathBuf) -> std::io::Result<Source> {
        let mut source = String::new();
        let mut file   = File::open(path.clone())?;
        file.read_to_string(&mut source)?;

        Ok(Source { contents: source, path })
    }

    pub fn source(source: &str) -> Source {
        Source { contents: source.to_string(), path: PathBuf::from("source") }
    }
}
