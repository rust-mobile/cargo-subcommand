use std::path::{Path, PathBuf};

use crate::manifest::CrateType;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ArtifactType {
    Lib,
    Bin,
    Example,
    // Bench,
    // Test,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Artifact {
    pub name: String,
    pub path: PathBuf,
    pub r#type: ArtifactType,
}

impl Artifact {
    pub fn build_dir(&self) -> &'static Path {
        Path::new(match self.r#type {
            ArtifactType::Lib | ArtifactType::Bin => "",
            ArtifactType::Example => "examples",
        })
    }

    // TODO: CrateType should be read from the manifest' crate-type array,
    // and validated that the requested format is in that array
    pub fn file_name(&self, ty: CrateType, target: &str) -> String {
        match (self.r#type, ty) {
            (ArtifactType::Bin | ArtifactType::Example, CrateType::Bin) => {
                if target.contains("windows") {
                    format!("{}.exe", self.name)
                } else if target.contains("wasm") {
                    format!("{}.wasm", self.name)
                } else {
                    self.name.to_string()
                }
            }
            (ArtifactType::Lib | ArtifactType::Example, CrateType::Lib) => {
                format!("lib{}.rlib", self.name.replace('-', "_"))
            }
            (ArtifactType::Lib | ArtifactType::Example, CrateType::Staticlib) => {
                format!("lib{}.a", self.name.replace('-', "_"))
            }
            (ArtifactType::Lib | ArtifactType::Example, CrateType::Cdylib) => {
                format!("lib{}.so", self.name.replace('-', "_"))
            }
            (a, c) => panic!("{a:?} is not compatible with {c:?}"),
        }
    }
}
