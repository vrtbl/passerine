use std::{fs, path::Path, rc::Rc};

use crate::source::Source;

pub struct Module {
    source: Rc<Source>,
    children: Vec<Module>,
}

pub const ENTRY_POINT: &str = "main";
pub const EXTENSION: &str = "pn";

// TODO: handle symlinks, ugh

impl Module {
    pub fn new_from_dir(entry_path: &Path) -> Result<Module, String> {
        // grab the entries in the directory
        let paths = fs::read_dir(entry_path).map_err(|_| {
            format!(
                "The path `{}` could not be read as a directory",
                entry_path.display()
            )
        })?;

        // incrementally build up the struct
        let entry_point_name = format!("{}.{}", ENTRY_POINT, EXTENSION);
        let mut entry: Option<Rc<Source>> = None;
        let mut children = vec![];

        for path in paths {
            // get an actual entry
            let path = path
                .map_err(|_| {
                    format!(
                        "Err while scanning the module located at `{}`",
                        entry_path.display()
                    )
                })?
                .path();

            // classify the entry
            let is_source_file = path.extension().map(|x| x == EXTENSION).unwrap_or(false);
            let is_entry_point = path.file_stem().map(|x| x == ENTRY_POINT).unwrap_or(false);

            // grab the module at the given path
            let module = if path.is_dir() {
                if path.join(&entry_point_name).exists() {
                    Module::new_from_dir(&path)?
                } else {
                    continue;
                }
            } else if path.is_file() && is_source_file {
                let source = Source::path(&path)
                    .map_err(|_| format!("Could not read source file `{}`", path.display()))?;

                if is_entry_point {
                    if let Some(other_path) = entry {
                        return Err(format!(
                            "Two potential entry points (`{}` and `{}`) for a single module",
                            other_path.path.display(),
                            path.display(),
                        ));
                    } else {
                        entry = Some(source);
                    }
                    continue;
                } else {
                    Module {
                        source,
                        children: Vec::new(),
                    }
                }
            } else {
                continue;
            };

            // append the module to the list of child modules
            children.push(module);
        }

        let source = entry.ok_or_else(|| {
            format!(
                "No entry point (e.g. `{}`) in the directory for the module `{}`",
                entry_point_name,
                entry_path.display()
            )
        })?;

        Ok(Module { source, children })
    }
}
