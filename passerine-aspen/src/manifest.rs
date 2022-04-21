use std::{
    fs::File,
    io::Read,
    path::Path,
};

use semver::Version;
use serde::{
    Deserialize,
    Serialize,
};
use toml::{
    self,
    map::Map,
};

use crate::MANIFEST;

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    package:      Package,
    dependencies: Map<String, toml::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    // required keys
    name:    String,      // package name
    version: String,      // package version, using semver
    authors: Vec<String>, // package authors

    // optional keys
    readme:        Option<String>, // path to package's readme
    license:       Option<String>, // Path to package's liscense
    repository:    Option<String>, // URL to package's repository
    documentation: Option<String>, // URL to package's documentation
}

impl Manifest {
    pub fn new(name: String) -> Manifest {
        Manifest {
            package:      Package {
                name,
                version: format!("{}", Version::new(0, 0, 0)),
                authors: vec![],
                readme: None,
                license: None,
                repository: None,
                documentation: None,
            },
            dependencies: Map::new(),
        }
    }

    pub fn package(mut path: &Path) -> Result<(Manifest, &Path), String> {
        let mut source = String::new();
        let mut file = None;

        // search up path for manifest
        while let None = file {
            match File::open(path.join(MANIFEST)) {
                Err(_) => {
                    path = path
                        .parent()
                        .ok_or("The manifest file could not be found")?;
                },
                Ok(f) => {
                    file = Some(f);
                },
            };
        }

        file.unwrap()
            .read_to_string(&mut source)
            .map_err(|_| "The manifest file could not be read")?;

        return Ok((
            Manifest::parse(&source)
                .ok_or("Could not parse the manifest file")?,
            path,
        ));
    }

    pub fn parse(source: &str) -> Option<Manifest> {
        // TODO: error handling
        toml::from_str(source).ok()
    }
}
