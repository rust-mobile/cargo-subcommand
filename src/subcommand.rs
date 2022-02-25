use crate::artifact::Artifact;
use crate::error::Error;
use crate::profile::Profile;
use crate::utils;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct Subcommand {
    cmd: String,
    args: Vec<String>,
    package: String,
    manifest: PathBuf,
    target_dir: PathBuf,
    target: Option<String>,
    host_triple: String,
    profile: Profile,
    artifacts: Vec<Artifact>,
    quiet: bool,
}

impl Subcommand {
    pub fn new<F: FnMut(&str, Option<&str>) -> Result<bool, Error>>(
        args: impl Iterator<Item = String>,
        subcommand: &'static str,
        mut parser: F,
    ) -> Result<Self, Error> {
        let mut args = args.peekable();
        args.next().ok_or(Error::InvalidArgs)?;
        let arg = args.next().ok_or(Error::InvalidArgs)?;
        if arg != subcommand {
            return Err(Error::InvalidArgs);
        }
        let cmd = args.next().unwrap_or_else(|| "--help".to_string());
        let mut cargo_args = Vec::new();
        let mut target = None;
        let mut profile = Profile::Dev;
        let mut artifacts = Vec::new();
        let mut target_dir = None;
        let mut manifest_path = None;
        let mut package = None;
        let mut examples = false;
        let mut bins = false;
        let mut quiet = false;
        while let Some(mut name) = args.next() {
            let value = if let Some(position) = name.as_str().find('=') {
                name.remove(position); // drop the '=' sign so we can cleanly split the string in two
                Some(name.split_off(position))
            } else if let Some(value) = args.peek() {
                if !value.starts_with('-') {
                    args.next()
                } else {
                    None
                }
            } else {
                None
            };
            let value_ref = value.as_deref();
            let mut matched = true;
            match (name.as_str(), value_ref) {
                ("--quiet", None) => quiet = true,
                ("--release", None) => profile = Profile::Release,
                ("--target", Some(value)) => target = Some(value.to_string()),
                ("--profile", Some("dev")) => profile = Profile::Dev,
                ("--profile", Some("release")) => profile = Profile::Release,
                ("--profile", Some(value)) => profile = Profile::Custom(value.to_string()),
                ("--example", Some(value)) => artifacts.push(Artifact::Example(value.to_string())),
                ("--examples", None) => examples = true,
                ("--bin", Some(value)) => artifacts.push(Artifact::Root(value.to_string())),
                ("--bins", None) => bins = true,
                ("--package", Some(value)) | ("-p", Some(value)) => {
                    package = Some(value.to_string())
                }
                ("--target-dir", Some(value)) => target_dir = Some(PathBuf::from(value)),
                ("--manifest-path", Some(value)) => manifest_path = Some(PathBuf::from(value)),
                _ => matched = false,
            }
            if matched || !parser(name.as_str(), value_ref)? {
                cargo_args.push(name);
                if let Some(value) = value {
                    cargo_args.push(value);
                }
            }
        }
        let (manifest, package) = utils::find_package(
            &manifest_path.unwrap_or_else(|| std::env::current_dir().unwrap()),
            package.as_deref(),
        )?;
        let root_dir = manifest.parent().unwrap();

        let target_dir = target_dir
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
            utils::find_workspace(&manifest, &package)
                .unwrap()
                .unwrap_or_else(|| manifest.clone())
                .parent()
                .unwrap()
                .join(utils::get_target_dir_name(root_dir).unwrap())
        });
        if examples {
            for file in utils::list_rust_files(&root_dir.join("examples"))? {
                artifacts.push(Artifact::Example(file));
            }
        }
        if bins {
            for file in utils::list_rust_files(&root_dir.join("src").join("bin"))? {
                artifacts.push(Artifact::Root(file));
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
        Ok(Self {
            cmd,
            args: cargo_args,
            package,
            manifest,
            target_dir,
            target,
            host_triple,
            profile,
            artifacts,
            quiet,
        })
    }

    pub fn cmd(&self) -> &str {
        &self.cmd
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn manifest(&self) -> &Path {
        &self.manifest
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
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
        self.quiet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separator_space() {
        let args = ["cargo", "subcommand", "build", "--target", "x86_64-unknown-linux-gnu"].iter().map(|s| s.to_string());
        let cmd = Subcommand::new(args, "subcommand", |_, _| Ok(false)).unwrap();
        assert_eq!(cmd.target(), Some("x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn test_separator_equals() {
        let args = ["cargo", "subcommand", "build", "--target=x86_64-unknown-linux-gnu"].iter().map(|s| s.to_string());
        let cmd = Subcommand::new(args, "subcommand", |_, _| Ok(false)).unwrap();
        assert_eq!(cmd.target(), Some("x86_64-unknown-linux-gnu"));
    }
}
