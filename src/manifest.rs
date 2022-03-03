use crate::error::Error;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub workspace: Option<Workspace>,
    pub package: Option<Package>,
}

impl Manifest {
    pub fn parse_from_toml(path: &Path) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::Io(path.to_owned(), e))?;
        toml::from_str(&contents).map_err(|e| Error::Toml(path.to_owned(), e))
    }
}

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub members: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
}
