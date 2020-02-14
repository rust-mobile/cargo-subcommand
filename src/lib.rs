use cargo_project::Project;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Profile {
    Dev,
    Release,
    Custom(String),
}

impl AsRef<Path> for Profile {
    fn as_ref(&self) -> &Path {
        Path::new(match self {
            Self::Dev => "debug",
            Self::Release => "release",
            Self::Custom(profile) => profile.as_str(),
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Artifact {
    Root(String),
    Example(String),
}

impl AsRef<Path> for Artifact {
    fn as_ref(&self) -> &Path {
        Path::new(match self {
            Self::Root(_) => "",
            Self::Example(_) => "examples",
        })
    }
}

impl Artifact {
    pub fn name(&self) -> &str {
        match self {
            Self::Root(name) => name,
            Self::Example(name) => name,
        }
    }

    pub fn file_name(&self, ty: CrateType, target: &str) -> String {
        match ty {
            CrateType::Bin => {
                if target.contains("windows") {
                    format!("{}.exe", self.name())
                } else if target.contains("wasm") {
                    format!("{}.wasm", self.name())
                } else {
                    self.name().to_string()
                }
            }
            CrateType::Lib => format!("lib{}.rlib", self.name()),
            CrateType::Staticlib => format!("lib{}.a", self.name()),
            CrateType::Cdylib => format!("lib{}.so", self.name()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CrateType {
    Bin,
    Lib,
    Staticlib,
    Cdylib,
}

pub struct Subcommand {
    cmd: String,
    args: Vec<String>,
    host_triple: String,
    target: Option<String>,
    project: Project,
    artifacts: Vec<Artifact>,
    profile: Profile,
    target_dir: PathBuf,
}

impl Subcommand {
    pub fn new(subcommand: &'static str) -> Result<Self, Error> {
        let mut args = std::env::args();
        let arg = args.next().ok_or(Error::InvalidArgs)?;
        if arg != "cargo" {
            log::warn!("Not run from cargo.");
        }
        let arg = args.next().ok_or(Error::InvalidArgs)?;
        if arg != subcommand {
            return Err(Error::InvalidArgs);
        }
        let cmd = args.next().ok_or(Error::InvalidArgs)?;
        let args: Vec<_> = args.collect();
        let mut target = None;
        let mut profile = Profile::Dev;
        let mut artifacts = Vec::new();
        let mut target_dir = None;
        let mut manifest_path = None;
        let mut package = None;
        let mut examples = false;
        let mut bins = false;
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--release" => profile = Profile::Release,
                "--target" => {
                    let arg = iter.next().ok_or(Error::InvalidArgs)?;
                    target = Some(arg.to_string());
                }
                "--profile" => match iter.next().ok_or(Error::InvalidArgs)?.as_str() {
                    "dev" => profile = Profile::Dev,
                    "release" => profile = Profile::Release,
                    arg => profile = Profile::Custom(arg.to_string()),
                },
                "--example" => {
                    let arg = iter.next().ok_or(Error::InvalidArgs)?;
                    artifacts.push(Artifact::Example(arg.to_string()));
                }
                "--examples" => examples = true,
                "--bin" => {
                    let arg = iter.next().ok_or(Error::InvalidArgs)?;
                    artifacts.push(Artifact::Root(arg.to_string()));
                }
                "--bins" => bins = true,
                "--package" | "-p" => {
                    let arg = iter.next().ok_or(Error::InvalidArgs)?;
                    package = Some(arg.to_string());
                }
                "--target-dir" => {
                    let arg = iter.next().ok_or(Error::InvalidArgs)?;
                    target_dir = Some(PathBuf::from(arg));
                }
                "--manifest-path" => {
                    let arg = iter.next().ok_or(Error::InvalidArgs)?;
                    manifest_path = Some(PathBuf::from(arg));
                }
                _ => {}
            }
        }
        let manifest_path = manifest_path.unwrap_or_else(|| std::env::current_dir().unwrap());
        // TODO project takes a package
        if package.is_some() {
            log::warn!("-p, --package option not implemented");
        }
        let project = Project::query(&manifest_path).map_err(|_| Error::ManifestNotFound)?;
        if artifacts.is_empty() {
            artifacts.push(Artifact::Root(project.name().replace("-", "_")));
        }
        let target_dir = target_dir.unwrap_or_else(|| project.target_dir().to_path_buf());
        // TODO examples and bins: add artifacts
        if examples {
            log::warn!("--examples option not implemented");
        }
        if bins {
            log::warn!("--bins option not implemented");
        }
        let host_triple = Command::new("rustc")
            .arg("-vV")
            .output()
            .map_err(|_| Error::RustcNotFound)?
            .stdout
            .lines()
            .map(|l| l.unwrap())
            .find(|l| l.starts_with("host: "))
            .map(|l| l[6..].to_string())
            .ok_or(Error::RustcNotFound)?;
        Ok(Self {
            cmd,
            args,
            host_triple,
            target,
            project,
            profile,
            artifacts,
            target_dir,
        })
    }

    pub fn cmd(&self) -> &str {
        &self.cmd
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn crate_name(&self) -> &str {
        self.project.name()
    }

    pub fn manifest(&self) -> &Path {
        self.project.toml()
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_ref().map(|s| &**s)
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn artifacts(&self) -> &[Artifact] {
        &self.artifacts
    }

    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    pub fn host_triple(&self) -> &str {
        &self.host_triple
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidArgs,
    ManifestNotFound,
    RustcNotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            Self::InvalidArgs => "Invalid args.",
            Self::ManifestNotFound => "Didn't find Cargo.toml",
            Self::RustcNotFound => "Didn't find rustc.",
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for Error {}
