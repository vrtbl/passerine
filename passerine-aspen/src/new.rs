use std::{
    fs,
    path::PathBuf,
};

use crate::{
    manifest::Manifest,
    status::{
        Kind,
        Status,
    },
    ENTRYPOINT,
    MANIFEST,
    SOURCE,
};

pub fn new(package: PathBuf) -> Result<(), String> {
    // get the name of the package
    // TODO: allow to be specified via argument
    let name = package
        .file_name()
        .ok_or("Can not determine directory name")?
        .to_str()
        .ok_or("Directory name is not representable")?
        .to_owned();

    // create the directory
    fs::create_dir_all(&package)
        .map_err(|_| "Unable to create package directory")?;

    if package.join(MANIFEST).is_file() {
        Status::warn().log(&format!(
            "The manifest file ({}) has already been created",
            MANIFEST
        ))
    } else {
        // write the manifest
        let manifest = Manifest::new(name.clone());
        fs::write(
            package.join(MANIFEST),
            toml::to_string_pretty(&manifest)
                .map_err(|_| "Could not generate manifest file")?,
        )
        .map_err(|_| "Could not write manifest file")?;
    }

    if package.join(SOURCE).is_dir() {
        Status::warn().log(&format!(
            "The source directory ({}/) has already been created",
            SOURCE
        ))
    } else {
        // create the source directory
        fs::create_dir(package.join(SOURCE))
            .map_err(|_| "Could not create source directory")?;
    }

    if package.join(SOURCE).join(ENTRYPOINT).is_file() {
        Status::warn().log(&format!(
            "The source entrypoint ({}/{}) has already been created",
            SOURCE, ENTRYPOINT
        ));
    } else {
        fs::write(
            package.join(SOURCE).join(ENTRYPOINT),
            "println \"Hello, Passerine!\"\n",
        )
        .map_err(|_| "Could not create source entrypoint")?;
    }

    Status(Kind::Success, "Finished")
        .log(&format!("The package '{}' was created successfully", name));
    Ok(())
}
