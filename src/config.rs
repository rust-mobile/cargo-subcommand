use crate::error::Error;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub build: Option<Build>,
}

impl Config {
    pub fn parse_from_toml(path: &Path) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|e| Error::Toml(path.to_owned(), e))
    }
}

#[derive(Debug, Deserialize)]
pub struct Build {
    #[serde(rename = "target-dir")]
    pub target_dir: Option<String>,
}
