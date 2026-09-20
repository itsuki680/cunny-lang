use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{Machine, compile};

pub struct BuildReport {
    pub pages: usize,
    pub output: PathBuf,
}

pub fn build_site(root: &Path) -> Result<BuildReport, String> {
    if !root.is_dir() {
        return Err(format!(
            "site directory '{}' does not exist",
            root.display()
        ));
    }

    let pages_directory = root.join("pages");
    if !pages_directory.is_dir() {
        return Err(format!(
            "site needs a pages directory at '{}'",
            pages_directory.display()
        ));
    }

    let mut pages = Vec::new();
    collect_pages(&pages_directory, &mut pages)?;
    pages.sort();
    if pages.is_empty() {
        return Err("the pages directory contains no .cunny files".to_owned());
    }

    let header = run_optional_layout(&root.join("layout/header.cunny"))?;
    let footer = run_optional_layout(&root.join("layout/footer.cunny"))?;
    let mut rendered_pages = Vec::new();

    for page in &pages {
        let relative = page
            .strip_prefix(&pages_directory)
            .map_err(|error| format!("could not resolve page path: {error}"))?;
        let mut html = Vec::new();

        html.extend_from_slice(&header);
        html.extend_from_slice(&run_program(page)?);
        html.extend_from_slice(&footer);

        rendered_pages.push((relative.with_extension("html"), html));
    }

    let assets = root.join("assets");
    if assets.exists() && !assets.is_dir() {
        return Err(format!("'{}' is not a directory", assets.display()));
    }

    let output = root.join("dist");
    prepare_output_directory(&output)?;

    for (relative, html) in rendered_pages {
        let destination = output.join(relative);

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("could not create '{}': {error}", parent.display()))?;
        }
        fs::write(&destination, html)
            .map_err(|error| format!("could not write '{}': {error}", destination.display()))?;
    }

    if assets.exists() {
        copy_directory(&assets, &output.join("assets"))?;
    }

    Ok(BuildReport {
        pages: pages.len(),
        output,
    })
}

fn collect_pages(directory: &Path, pages: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("could not read '{}': {error}", directory.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("could not read directory entry: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("could not inspect '{}': {error}", entry.path().display()))?;

        if file_type.is_dir() {
            collect_pages(&entry.path(), pages)?;
        } else if file_type.is_file() && entry.path().extension() == Some(OsStr::new("cunny")) {
            pages.push(entry.path());
        }
    }

    Ok(())
}

fn run_optional_layout(path: &Path) -> Result<Vec<u8>, String> {
    if path.exists() {
        run_program(path)
    } else {
        Ok(Vec::new())
    }
}

fn run_program(path: &Path) -> Result<Vec<u8>, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("could not read '{}': {error}", path.display()))?;
    let instructions = compile(&source).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut machine = Machine::new();
    let mut input = io::empty();
    let mut output = Vec::new();
    machine
        .run(&instructions, &mut input, &mut output)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(output)
}

fn prepare_output_directory(output: &Path) -> Result<(), String> {
    if output.exists() {
        let metadata = fs::symlink_metadata(output)
            .map_err(|error| format!("could not inspect '{}': {error}", output.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "refusing to replace '{}': it is not a normal directory",
                output.display()
            ));
        }
        fs::remove_dir_all(output)
            .map_err(|error| format!("could not clean '{}': {error}", output.display()))?;
    }

    fs::create_dir_all(output)
        .map_err(|error| format!("could not create '{}': {error}", output.display()))
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("could not create '{}': {error}", destination.display()))?;

    let entries = fs::read_dir(source)
        .map_err(|error| format!("could not read '{}': {error}", source.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("could not read directory entry: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("could not inspect '{}': {error}", entry.path().display()))?;
        let target = destination.join(entry.file_name());

        if file_type.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &target)
                .map_err(|error| format!("could not copy '{}': {error}", entry.path().display()))?;
        } else {
            return Err(format!(
                "assets cannot contain symbolic links: '{}'",
                entry.path().display()
            ));
        }
    }

    Ok(())
}
