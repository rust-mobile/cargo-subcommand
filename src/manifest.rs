use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::error::{Error, Result};
use crate::utils;

#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    pub workspace: Option<Workspace>,
    pub package: Option<Package>,
    pub lib: Option<Lib>,
    #[serde(default, rename = "bin")]
    pub bins: Vec<Bin>,
    #[serde(default, rename = "example")]
    pub examples: Vec<Example>,
}

impl Manifest {
    pub fn parse_from_toml(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::Io(path.to_owned(), e))?;
        toml::from_str(&contents).map_err(|e| Error::Toml(path.to_owned(), e))
    }

    /// Returns a mapping from manifest directory to manifest path and loaded manifest
    pub fn members(&self, workspace_root: &Path) -> Result<HashMap<PathBuf, (PathBuf, Manifest)>> {
        let workspace = self
            .workspace
            .as_ref()
            .ok_or(Error::ManifestNotAWorkspace)?;
        let workspace_root = utils::canonicalize(workspace_root)?;

        // Check all member packages inside the workspace
        let mut all_members = HashMap::new();

        for member in &workspace.members {
            for manifest_dir in glob::glob(workspace_root.join(member).to_str().unwrap())? {
                let manifest_dir = manifest_dir?;
                let manifest_path = manifest_dir.join("Cargo.toml");
                let manifest = Manifest::parse_from_toml(&manifest_path)?;

                // Workspace members cannot themselves be/contain a new workspace
                if manifest.workspace.is_some() {
                    return Err(Error::UnexpectedWorkspace(manifest_path));
                }

                // And because they cannot contain a [workspace], they may not be a virtual manifest
                // and must hence contain [package]
                if manifest.package.is_none() {
                    return Err(Error::NoPackageInManifest(manifest_path));
                }

                all_members.insert(manifest_dir, (manifest_path, manifest));
            }
        }

        Ok(all_members)
    }

    /// Returns `self` if it contains `[package]` but not `[workspace]`, (i.e. it cannot be
    /// a workspace nor a virtual manifest), and describes a package named `name` if not [`None`].
    pub fn map_nonvirtual_package(
        self,
        manifest_path: PathBuf,
        name: Option<&str>,
    ) -> Result<(PathBuf, Self)> {
        if self.workspace.is_some() {
            return Err(Error::UnexpectedWorkspace(manifest_path));
        }

        if let Some(package) = &self.package {
            if let Some(name) = name {
                if package.name == name {
                    Ok((manifest_path, self))
                } else {
                    Err(Error::PackageNotFound(manifest_path, name.into()))
                }
            } else {
                Ok((manifest_path, self))
            }
        } else {
            Err(Error::NoPackageInManifest(manifest_path))
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Workspace {
    #[serde(default)]
    pub default_members: Vec<String>,
    #[serde(default)]
    pub members: Vec<String>,
}

const fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct Package {
    pub name: String,

    // https://doc.rust-lang.org/cargo/reference/cargo-targets.html#target-auto-discovery
    #[serde(default = "default_true")]
    pub autobins: bool,
    #[serde(default = "default_true")]
    pub autoexamples: bool,
    // #[serde(default = "default_true")]
    // pub autotests: bool,
    // #[serde(default = "default_true")]
    // pub autobenches: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize)]
pub enum CrateType {
    Bin,
    Lib,
    Staticlib,
    Cdylib,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Lib {
    pub name: Option<String>,
    pub path: Option<PathBuf>,
    // pub crate_type: Vec<CrateType>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Bin {
    pub name: String,
    pub path: Option<PathBuf>,
    // pub crate_type: Vec<CrateType>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Example {
    pub name: String,
    pub path: Option<PathBuf>,
    // pub crate_type: Vec<CrateType>,
}
