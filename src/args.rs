use crate::profile::Profile;
#[cfg(feature = "clap")]
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "clap", derive(Parser))]
pub struct Args {
    /// No output printed to stdout
    #[cfg_attr(feature = "clap", clap(long, short))]
    pub quiet: bool,

    /// Package to build
    #[cfg_attr(feature = "clap", clap(long, short))]
    pub package: Vec<String>,
    /// Build all packages in the workspace
    #[cfg_attr(feature = "clap", clap(long))]
    pub workspace: bool,
    /// Exclude packages from the build
    #[cfg_attr(feature = "clap", clap(long))]
    pub exclude: Vec<String>,

    /// Build only this package's library
    #[cfg_attr(feature = "clap", clap(long))]
    pub lib: bool,
    /// Build only the specified binary
    #[cfg_attr(feature = "clap", clap(long))]
    pub bin: Vec<String>,
    /// Build all binaries
    #[cfg_attr(feature = "clap", clap(long, conflicts_with = "bin"))]
    pub bins: bool,
    /// Build only the specified example
    #[cfg_attr(feature = "clap", clap(long))]
    pub example: Vec<String>,
    /// Build all examples
    #[cfg_attr(feature = "clap", clap(long, conflicts_with = "example"))]
    pub examples: bool,

    /// Build artifacts in release mode, with optimizations
    #[cfg_attr(feature = "clap", clap(long))]
    pub release: bool,
    /// Build artifacts with the specified profile
    #[cfg_attr(feature = "clap", clap(long, conflicts_with = "release"))]
    pub profile: Option<Profile>,
    /// Space or comma separated list of features to activate
    #[cfg_attr(feature = "clap", clap(long))]
    pub features: Vec<String>,
    /// Activate all available features
    #[cfg_attr(feature = "clap", clap(long))]
    pub all_features: bool,
    /// Do not activate the `default` feature
    #[cfg_attr(feature = "clap", clap(long))]
    pub no_default_features: bool,
    /// Build for the target triple
    #[cfg_attr(feature = "clap", clap(long))]
    pub target: Option<String>,
    /// Directory for all generated artifacts
    #[cfg_attr(feature = "clap", clap(long))]
    pub target_dir: Option<PathBuf>,
    /// Path to Cargo.toml
    #[cfg_attr(feature = "clap", clap(long))]
    pub manifest_path: Option<PathBuf>,
}

impl Args {
    pub fn apply(&self, cmd: &mut Command) {
        if self.quiet {
            cmd.arg("--quiet");
        }
        for package in &self.package {
            cmd.arg("--package").arg(package);
        }
        if self.workspace {
            cmd.arg("--workspace");
        }
        for exclude in &self.exclude {
            cmd.arg("--exclude").arg(exclude);
        }

        if self.lib {
            cmd.arg("--lib");
        }
        for bin in &self.bin {
            cmd.arg("--bin").arg(bin);
        }
        if self.bins {
            cmd.arg("--bins");
        }
        for example in &self.example {
            cmd.arg("--example").arg(example);
        }
        if self.examples {
            cmd.arg("--examples");
        }

        if self.release {
            cmd.arg("--release");
        }
        if let Some(profile) = self.profile.as_ref() {
            cmd.arg("--profile").arg(profile.to_string());
        }
        for features in &self.features {
            cmd.arg("--features").arg(features);
        }
        if self.all_features {
            cmd.arg("--all-features");
        }
        if self.no_default_features {
            cmd.arg("--no-default-features");
        }
        if let Some(target) = self.target.as_ref() {
            cmd.arg("--target").arg(target);
        }
        if let Some(target_dir) = self.target_dir.as_ref() {
            cmd.arg("--target-dir").arg(target_dir);
        }
        if let Some(manifest_path) = self.manifest_path.as_ref() {
            cmd.arg("--manifest-path").arg(manifest_path);
        }
    }

    pub fn profile(&self) -> Profile {
        if let Some(profile) = self.profile.as_ref() {
            profile.clone()
        } else if self.release {
            Profile::Release
        } else {
            Profile::Dev
        }
    }
}
