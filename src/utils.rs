use crate::config::Config;
use crate::error::{Error, Result};
use crate::manifest::Manifest;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn list_rust_files(dir: &Path) -> Result<Vec<String>> {
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

/// Tries to find a package by the given `name` in the [workspace root] or member
/// of the given [workspace] [`Manifest`].
///
/// When a workspace is not detected, call [`find_package_manifest()`] instead.
///
/// [workspace root]: https://doc.rust-lang.org/cargo/reference/workspaces.html#root-package
/// [workspace]: https://doc.rust-lang.org/cargo/reference/workspaces.html#workspaces
pub fn find_package_manifest_in_workspace(
    workspace_manifest_path: &Path,
    workspace_manifest: &Manifest,
    name: &str,
) -> Result<(PathBuf, Manifest)> {
    let workspace = workspace_manifest
        .workspace
        .as_ref()
        .ok_or(Error::ManifestNotAWorkspace)?;

    // Check if the workspace manifest also contains a [package]
    if let Some(package) = &workspace_manifest.package {
        if package.name == name {
            return Ok((
                workspace_manifest_path.to_owned(),
                workspace_manifest.clone(),
            ));
        }
    }

    // Check all member packages inside the workspace
    let workspace_root = workspace_manifest_path.parent().unwrap();
    for member in &workspace.members {
        for manifest_dir in glob::glob(workspace_root.join(member).to_str().unwrap())? {
            let manifest_path = manifest_dir?.join("Cargo.toml");
            let manifest = Manifest::parse_from_toml(&manifest_path)?;

            // Workspace members cannot themselves be/contain a new workspace
            if manifest.workspace.is_some() {
                return Err(Error::UnexpectedWorkspace(manifest_path));
            }

            if let Some(package) = &manifest.package {
                if package.name == name {
                    return Ok((manifest_path, manifest));
                }
            } else {
                return Err(Error::NoPackageInManifest(manifest_path));
            }
        }
    }

    Err(Error::PackageNotFound(
        workspace_manifest_path.into(),
        name.into(),
    ))
}

/// Recursively walk up the directories until finding a `Cargo.toml`
///
/// When a workspace has been detected, use [`find_package_manifest_in_workspace()`] to find packages
/// instead (that are members of the given workspace) when the user specified a package name (with `-p`).
pub fn find_package_manifest(path: &Path, name: Option<&str>) -> Result<(PathBuf, Manifest)> {
    let path = dunce::canonicalize(path).map_err(|e| Error::Io(path.to_owned(), e))?;
    let manifest_path = path
        .ancestors()
        .map(|dir| dir.join("Cargo.toml"))
        .find(|manifest| manifest.exists())
        .ok_or(Error::ManifestNotFound)?;

    let manifest = Manifest::parse_from_toml(&manifest_path)?;

    // This function shouldn't be called when a workspace exists.
    if manifest.workspace.is_some() {
        return Err(Error::UnexpectedWorkspace(manifest_path));
    }

    if let Some(package) = &manifest.package {
        if let Some(name) = name {
            if package.name == name {
                Ok((manifest_path, manifest))
            } else {
                Err(Error::PackageNotFound(manifest_path, name.into()))
            }
        } else {
            Ok((manifest_path, manifest))
        }
    } else {
        Err(Error::NoPackageInManifest(manifest_path))
    }
}

/// Find the first `Cargo.toml` that contains a `[workspace]`
pub fn find_workspace(potential_root: &Path) -> Result<Option<(PathBuf, Manifest)>> {
    for manifest_path in potential_root
        .ancestors()
        .map(|dir| dir.join("Cargo.toml"))
        .filter(|manifest| manifest.exists())
    {
        let manifest = Manifest::parse_from_toml(&manifest_path)?;
        if manifest.workspace.is_some() {
            return Ok(Some((manifest_path, manifest)));
        }
    }
    Ok(None)
}

/// Returns the [`target-dir`] configured in `.cargo/config.toml` or `"target"` if not set.
///
/// [`target-dir`]: https://doc.rust-lang.org/cargo/reference/config.html#buildtarget-dir
pub fn get_target_dir_name(config: Option<&Config>) -> Result<String> {
    if let Some(config) = config {
        if let Some(build) = config.build.as_ref() {
            if let Some(target_dir) = &build.target_dir {
                return Ok(target_dir.clone());
            }
        }
    }
    Ok("target".to_string())
}
