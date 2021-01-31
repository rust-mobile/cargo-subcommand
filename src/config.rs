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
        Ok(toml::from_str(&contents)?)
    }
}

#[derive(Debug, Deserialize)]
pub struct Build {
    #[serde(rename = "target-dir")]
    pub target_dir: Option<String>,
}
