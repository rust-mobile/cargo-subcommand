use crate::args::Args;
use crate::artifact::{Artifact, CrateType};
use crate::error::Error;
use crate::profile::Profile;
use crate::{utils, LocalizedConfig};
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct Subcommand {
    args: Args,
    package: String,
    manifest: PathBuf,
    target_dir: PathBuf,
    host_triple: String,
    profile: Profile,
    artifacts: Vec<Artifact>,
    config: Option<LocalizedConfig>,
}

impl Subcommand {
    pub fn new(args: Args) -> Result<Self, Error> {
        let (manifest_path, package) = utils::find_package(
            &args
                .manifest_path
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap()),
            args.package.as_deref(),
        )?;
        let root_dir = manifest_path.parent().unwrap();

        let target_dir = args
            .target_dir
            .clone()
            .or_else(|| {
                std::env::var_os("CARGO_BUILD_TARGET_DIR")
                    .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
                    .map(|os_str| os_str.into())
            })
            .map(|target_dir| {
                if target_dir.is_relative() {
                    std::env::current_dir().unwrap().join(target_dir)
                } else {
                    target_dir
                }
            });

        // TODO: Find, parse, and merge _all_ config files following the hierarchical structure:
        // https://doc.rust-lang.org/cargo/reference/config.html#hierarchical-structure
        let config = LocalizedConfig::find_cargo_config_for_workspace(&root_dir)?;
        if let Some(config) = &config {
            config.set_env_vars().unwrap();
        }

        let target_dir = target_dir.unwrap_or_else(|| {
            utils::find_workspace(&manifest_path, &package)
                .unwrap()
                .unwrap_or_else(|| manifest_path.clone())
                .parent()
                .unwrap()
                .join(utils::get_target_dir_name(config.as_deref()).unwrap())
        });

        let mut artifacts = vec![];
        if args.examples {
            for file in utils::list_rust_files(&root_dir.join("examples"))? {
                artifacts.push(Artifact::Example(file));
            }
        } else {
            for example in &args.example {
                artifacts.push(Artifact::Example(example.into()));
            }
        }
        if args.bins {
            for file in utils::list_rust_files(&root_dir.join("src").join("bin"))? {
                artifacts.push(Artifact::Root(file));
            }
        } else {
            for bin in &args.bin {
                artifacts.push(Artifact::Root(bin.into()));
            }
        }
        if artifacts.is_empty() {
            artifacts.push(Artifact::Root(package.clone()));
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
        let profile = args.profile();
        Ok(Self {
            args,
            package,
            manifest: manifest_path,
            target_dir,
            host_triple,
            profile,
            artifacts,
            config,
        })
    }

    pub fn args(&self) -> &Args {
        &self.args
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn manifest(&self) -> &Path {
        &self.manifest
    }

    pub fn target(&self) -> Option<&str> {
        self.args.target.as_deref()
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

    pub fn quiet(&self) -> bool {
        self.args.quiet
    }

    pub fn config(&self) -> Option<&LocalizedConfig> {
        self.config.as_ref()
    }

    pub fn build_dir(&self, target: Option<&str>) -> PathBuf {
        let target_dir = dunce::simplified(self.target_dir()).to_path_buf();
        let arch_dir = if let Some(target) = target {
            target_dir.join(target)
        } else {
            target_dir
        };
        arch_dir.join(self.profile())
    }

    pub fn artifact(
        &self,
        artifact: &Artifact,
        target: Option<&str>,
        crate_type: CrateType,
    ) -> PathBuf {
        let triple = target.unwrap_or_else(|| self.host_triple());
        let file_name = artifact.file_name(crate_type, triple);
        self.build_dir(target).join(artifact).join(file_name)
    }
}
