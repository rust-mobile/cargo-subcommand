use crate::args::Args;
use crate::artifact::{Artifact, ArtifactType};
use crate::error::{Error, Result};
use crate::manifest::Manifest;
use crate::profile::Profile;
use crate::{utils, CrateType, LocalizedConfig};
use std::collections::HashMap;
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
    lib_artifact: Option<Artifact>,
    bin_artifacts: Vec<Artifact>,
    example_artifacts: Vec<Artifact>,
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

        // Scan up the directories based on --manifest-path and the working directory to find a Cargo.toml
        let potential_manifest = utils::find_manifest(&search_path)?;
        // Perform the same scan, but for a Cargo.toml containing [workspace]
        let workspace_manifest = utils::find_workspace(&search_path)?;

        let (manifest_path, manifest) = {
            if let Some(workspace_manifest) = &workspace_manifest {
                utils::find_package_manifest_in_workspace(
                    workspace_manifest,
                    potential_manifest,
                    package,
                )?
            } else {
                let (manifest_path, manifest) = potential_manifest;
                manifest.map_nonvirtual_package(manifest_path, package)?
            }
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

        let parsed_manifest = Manifest::parse_from_toml(&manifest_path)?;

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

        // https://doc.rust-lang.org/cargo/reference/cargo-targets.html#target-auto-discovery

        let main_bin_path = Path::new("src/main.rs");
        let main_lib_path = Path::new("src/lib.rs");

        let mut bin_artifacts = HashMap::new();
        let mut example_artifacts = HashMap::new();

        fn find_main_file(dir: &Path, name: &str) -> Option<PathBuf> {
            let alt_path = dir.join(format!("{}.rs", name));
            alt_path.is_file().then_some(alt_path).or_else(|| {
                let alt_path = dir.join(name).join("main.rs");
                alt_path.is_file().then_some(alt_path)
            })
        }

        // Add all explicitly configured binaries
        for bin in &parsed_manifest.bins {
            let path = bin
                .path
                .clone()
                .or_else(|| find_main_file(&root_dir.join("src/bin"), &bin.name))
                .ok_or_else(|| Error::BinNotFound(bin.name.clone()))?;

            let prev = bin_artifacts.insert(
                bin.name.clone(),
                Artifact {
                    name: bin.name.clone(),
                    path,
                    r#type: ArtifactType::Bin,
                },
            );
            if prev.is_some() {
                return Err(Error::DuplicateBin(bin.name.clone()));
            }
        }

        // Add all explicitly configured examples
        for example in &parsed_manifest.examples {
            let path = example
                .path
                .clone()
                .or_else(|| find_main_file(&root_dir.join("examples"), &example.name))
                .ok_or_else(|| Error::ExampleNotFound(example.name.clone()))?;

            let prev = example_artifacts.insert(
                example.name.clone(),
                Artifact {
                    name: example.name.clone(),
                    path,
                    r#type: ArtifactType::Example,
                },
            );
            if prev.is_some() {
                return Err(Error::DuplicateExample(example.name.clone()));
            }
        }

        /// Name is typically the [`Path::file_stem()`], except for `src/main.rs` where it is the package name
        fn insert_if_unconfigured(
            name: Option<String>,
            path: &Path,
            r#type: ArtifactType,
            artifacts: &mut HashMap<String, Artifact>,
        ) {
            // Only insert the detected binary if there isn't another artifact already configuring this file path
            if artifacts.values().any(|bin| bin.path == path) {
                println!("Already configuring {path:?}");
                return;
            }

            let name =
                name.unwrap_or_else(|| path.file_stem().unwrap().to_str().unwrap().to_owned());

            // Only insert the detected binary if an artifact with the same name wasn't yet configured
            artifacts.entry(name.clone()).or_insert(Artifact {
                name,
                path: path.to_owned(),
                r#type,
            });
        }

        // Parse all autobins
        if parsed_manifest
            .package
            .as_ref()
            .map_or(true, |p| p.autobins)
        {
            // Special-case for the main binary of a package
            if root_dir.join(main_bin_path).is_file() {
                insert_if_unconfigured(
                    Some(package.clone()),
                    main_bin_path,
                    ArtifactType::Bin,
                    &mut bin_artifacts,
                );
            }

            for file in utils::list_rust_files(&root_dir.join("src").join("bin"))? {
                let file = file.strip_prefix(root_dir).unwrap();

                insert_if_unconfigured(None, file, ArtifactType::Bin, &mut bin_artifacts);
            }
        }

        // Parse all autoexamples
        if parsed_manifest
            .package
            .as_ref()
            .map_or(true, |p| p.autoexamples)
        {
            for file in utils::list_rust_files(&root_dir.join("examples"))? {
                let file = file.strip_prefix(root_dir).unwrap();

                insert_if_unconfigured(None, file, ArtifactType::Example, &mut example_artifacts);
            }
        }

        let mut lib_artifact = parsed_manifest
            .lib
            .as_ref()
            .map(|lib| Artifact {
                // The library is either configured with sensible defaults
                name: lib.name.as_ref().unwrap_or(package).clone(),
                path: lib.path.as_deref().unwrap_or(main_lib_path).to_owned(),
                r#type: ArtifactType::Lib,
            })
            .or_else(|| {
                // Or autodetected with the same defaults, if that default path exists
                root_dir.join(main_lib_path).is_file().then(|| Artifact {
                    name: package.clone(),
                    path: main_lib_path.to_owned(),
                    r#type: ArtifactType::Lib,
                })
            });

        // Filtering based on arguments
        // https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries

        let specific_target_selected = args.specific_target_selected();

        if specific_target_selected {
            if !args.lib {
                lib_artifact = None;
            }

            if !args.bins {
                bin_artifacts.retain(|a, _| args.bin.contains(a));
            }

            if !args.examples {
                example_artifacts.retain(|a, _| args.example.contains(a));
            }
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
            lib_artifact,
            bin_artifacts: bin_artifacts.into_values().collect(),
            example_artifacts: example_artifacts.into_values().collect(),
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

    pub fn artifacts(&self) -> impl Iterator<Item = &Artifact> {
        self.lib_artifact
            .iter()
            .chain(&self.bin_artifacts)
            .chain(&self.example_artifacts)
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
        self.build_dir(target)
            .join(artifact.build_dir())
            .join(file_name)
    }
}
