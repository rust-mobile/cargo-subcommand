use crate::args::Args;
use crate::artifact::{Artifact, CrateType};
use crate::error::{Error, Result};
use crate::profile::Profile;
use crate::{utils, LocalizedConfig};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Subcommand {
    args: Args,
    package: String,
    workspace_manifest: Option<PathBuf>,
    manifest: PathBuf,
    target_dir: PathBuf,
    host_triple: String,
    profile: Profile,
    artifacts: Vec<Artifact>,
    config: Option<LocalizedConfig>,
}

impl Subcommand {
    pub fn new(args: Args) -> Result<Self> {
        // TODO: support multiple packages properly
        assert!(
            args.package.len() < 2,
            "Multiple packages are not supported yet by `cargo-subcommand`"
        );
        let package = args.package.get(0).map(|s| s.as_str());
        assert!(
            !args.workspace,
            "`--workspace` is not supported yet by `cargo-subcommand`"
        );
        assert!(
            args.exclude.is_empty(),
            "`--exclude` is not supported yet by `cargo-subcommand`"
        );

        let manifest_path = args
            .manifest_path
            .clone()
            .map(|path| {
                if path.file_name() != Some(OsStr::new("Cargo.toml")) || !path.is_file() {
                    Err(Error::ManifestPathNotFound)
                } else {
                    Ok(path)
                }
            })
            .transpose()?;

        let search_path = manifest_path.map_or_else(
            || std::env::current_dir().map_err(|e| Error::Io(PathBuf::new(), e)),
            |manifest_path| utils::canonicalize(manifest_path.parent().unwrap()),
        )?;

        // Scan the given and all parent directories for a Cargo.toml containing a workspace
        let workspace_manifest = utils::find_workspace(&search_path)?;

        let (manifest_path, manifest) =
            if let (Some(package), Some((workspace_manifest_path, workspace))) =
                (package, &workspace_manifest)
            {
                // If a workspace was found, and the user chose a package with `-p`, find packages relative to it
                // TODO: What if we call `cargo apk run` in the workspace root, and detect a workspace? It should
                // then use the `[package]` defined in the workspace (will be found below, though, but currently
                // fails with UnexpectedWorkspace)
                utils::find_package_manifest_in_workspace(
                    workspace_manifest_path,
                    workspace,
                    package,
                )?
            } else {
                // Otherwise scan up the directories based on --manifest-path and the working directory.
                // TODO: When we're in a workspace but the user didn't select a package by name, this
                // is the right logic to use as long as we _also_ validate that the Cargo.toml we found
                // was a member of this workspace?
                utils::find_package_manifest(&search_path, package)?
            };

        // The manifest is known to contain a package at this point
        let package = &manifest.package.as_ref().unwrap().name;

        let root_dir = manifest_path.parent().unwrap();

        // TODO: Find, parse, and merge _all_ config files following the hierarchical structure:
        // https://doc.rust-lang.org/cargo/reference/config.html#hierarchical-structure
        let config = LocalizedConfig::find_cargo_config_for_workspace(root_dir)?;
        if let Some(config) = &config {
            config.set_env_vars().unwrap();
        }

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

        let target_dir = target_dir.unwrap_or_else(|| {
            workspace_manifest
                .as_ref()
                .map(|(path, _)| path)
                .unwrap_or_else(|| &manifest_path)
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
        let host_triple = current_platform::CURRENT_PLATFORM.to_owned();
        let profile = args.profile();
        Ok(Self {
            args,
            package: package.clone(),
            workspace_manifest: workspace_manifest.map(|(path, _)| path),
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

    pub fn workspace_manifest(&self) -> Option<&Path> {
        self.workspace_manifest.as_deref()
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
        let target_dir = dunce::simplified(self.target_dir());
        let arch_dir = if let Some(target) = target {
            target_dir.join(target)
        } else {
            target_dir.to_path_buf()
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
