use crate::config::Config;
use crate::error::Error;
use crate::manifest::Manifest;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn list_rust_files(dir: &Path) -> Result<Vec<String>, Error> {
    let mut files = Vec::new();
    if dir.exists() && dir.is_dir() {
        let entries = std::fs::read_dir(dir).map_err(|e| Error::Io(dir.to_owned(), e))?;
        for entry in entries {
            let path = entry.map_err(|e| Error::Io(dir.to_owned(), e))?.path();
            if path.is_file() && path.extension() == Some(OsStr::new("rs")) {
                let name = path.file_stem().unwrap().to_str().unwrap();
                files.push(name.to_string());
            }
        }
    }
    Ok(files)
}

fn member(manifest: &Path, members: &[String], package: &str) -> Result<Option<PathBuf>, Error> {
    let workspace_dir = manifest.parent().unwrap();
    for member in members {
        for manifest_dir in glob::glob(workspace_dir.join(member).to_str().unwrap())? {
            let manifest_path = manifest_dir?.join("Cargo.toml");
            let manifest = Manifest::parse_from_toml(&manifest_path)?;
            if let Some(p) = manifest.package.as_ref() {
                if p.name == package {
                    return Ok(Some(manifest_path));
                }
            }
        }
    }
    Ok(None)
}

pub fn find_package(path: &Path, name: Option<&str>) -> Result<(PathBuf, String), Error> {
    let path = dunce::canonicalize(path).map_err(|e| Error::Io(path.to_owned(), e))?;
    for manifest_path in path
        .ancestors()
        .map(|dir| dir.join("Cargo.toml"))
        .filter(|dir| dir.exists())
    {
        let manifest = Manifest::parse_from_toml(&manifest_path)?;
        if let Some(p) = manifest.package.as_ref() {
            if let (Some(n1), n2) = (name, &p.name) {
                if n1 == n2 {
                    return Ok((manifest_path, p.name.clone()));
                }
            } else {
                return Ok((manifest_path, p.name.clone()));
            }
        }
        if let (Some(w), Some(name)) = (manifest.workspace.as_ref(), name) {
            if let Some(manifest_path) = member(&manifest_path, &w.members, name)? {
                return Ok((manifest_path, name.to_string()));
            }
        }
    }
    Err(Error::ManifestNotFound)
}

pub fn find_workspace(manifest: &Path, name: &str) -> Result<Option<PathBuf>, Error> {
    let dir = manifest.parent().unwrap();
    for manifest_path in dir
        .ancestors()
        .map(|dir| dir.join("Cargo.toml"))
        .filter(|dir| dir.exists())
    {
        let manifest = Manifest::parse_from_toml(&manifest_path)?;
        if let Some(w) = manifest.workspace.as_ref() {
            if member(&manifest_path, &w.members, name)?.is_some() {
                return Ok(Some(manifest_path));
            }
        }
    }
    Ok(None)
}

/// Returns the [`target-dir`] configured in `.cargo/config.toml` or `"target"` if not set.
///
/// [`target-dir`](https://doc.rust-lang.org/cargo/reference/config.html#buildtarget-dir)
pub fn get_target_dir_name(config: Option<&Config>) -> Result<String, Error> {
    if let Some(config) = config {
        if let Some(build) = config.build.as_ref() {
            if let Some(target_dir) = &build.target_dir {
                return Ok(target_dir.clone());
            }
        }
    }
    Ok("target".to_string())
}
