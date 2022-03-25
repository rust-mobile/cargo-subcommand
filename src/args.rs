use crate::profile::Profile;
#[cfg(feature = "clap")]
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "clap", derive(Parser))]
pub struct Args {
    #[cfg_attr(feature = "clap", clap(long, short))]
    pub quiet: bool,
    #[cfg_attr(feature = "clap", clap(long, short))]
    pub release: bool,
    #[cfg_attr(feature = "clap", clap(long))]
    pub target: Option<String>,
    #[cfg_attr(feature = "clap", clap(long, conflicts_with = "release"))]
    pub profile: Option<Profile>,
    #[cfg_attr(feature = "clap", clap(long))]
    pub example: Vec<String>,
    #[cfg_attr(feature = "clap", clap(long, conflicts_with = "example"))]
    pub examples: bool,
    #[cfg_attr(feature = "clap", clap(long))]
    pub bin: Vec<String>,
    #[cfg_attr(feature = "clap", clap(long, conflicts_with = "bin"))]
    pub bins: bool,
    #[cfg_attr(feature = "clap", clap(long, short))]
    pub package: Option<String>,
    #[cfg_attr(feature = "clap", clap(long))]
    pub target_dir: Option<PathBuf>,
    #[cfg_attr(feature = "clap", clap(long))]
    pub manifest_path: Option<PathBuf>,
}

impl Args {
    pub fn apply(&self, cmd: &mut Command) {
        if self.quiet {
            cmd.arg("--quiet");
        }
        if self.release {
            cmd.arg("--release");
        }
        if let Some(target) = self.target.as_ref() {
            cmd.arg("--target").arg(target);
        }
        if let Some(profile) = self.profile.as_ref() {
            cmd.arg("--profile").arg(profile.to_string());
        }
        for example in &self.example {
            cmd.arg("--example").arg(example);
        }
        if self.examples {
            cmd.arg("--examples");
        }
        for bin in &self.bin {
            cmd.arg("--bin").arg(bin);
        }
        if self.bins {
            cmd.arg("--bins");
        }
        if let Some(package) = self.package.as_ref() {
            cmd.arg("--package").arg(package);
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
