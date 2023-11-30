use glob::{GlobError, PatternError};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::PathBuf;
use toml::de::Error as TomlError;

#[derive(Debug)]
pub enum Error {
    InvalidArgs,
    ManifestNotAWorkspace,
    ManifestNotFound,
    RustcNotFound,
    ManifestPathNotFound,
    GlobPatternError(&'static str),
    Glob(GlobError),
    UnexpectedWorkspace(PathBuf),
    NoPackageInManifest(PathBuf),
    PackageNotFound(PathBuf, String),
    ManifestNotInWorkspace {
        manifest: PathBuf,
        workspace_manifest: PathBuf,
    },
    Io(PathBuf, IoError),
    Toml(PathBuf, TomlError),
    BinNotFound(String),
    ExampleNotFound(String),
    DuplicateBin(String),
    DuplicateExample(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(match self {
            Self::InvalidArgs => "Invalid args.",
            Self::ManifestNotAWorkspace => {
                "The provided Cargo.toml does not contain a `[workspace]`"
            }
            Self::ManifestNotFound => "Didn't find Cargo.toml.",
            Self::ManifestPathNotFound => "The manifest-path must be a path to a Cargo.toml file",
            Self::RustcNotFound => "Didn't find rustc.",
            Self::GlobPatternError(error) => error,
            Self::Glob(error) => return error.fmt(f),
            Self::UnexpectedWorkspace(path) => {
                return write!(f, "Did not expect a `[workspace]` at `{}`", path.display())
            }
            Self::NoPackageInManifest(manifest) => {
                return write!(
                    f,
                    "Failed to parse manifest at `{}`: virtual manifests must be configured with `[workspace]`",
                    manifest.display()
                )
            }
            Self::PackageNotFound(workspace, name) => {
                return write!(
                    f,
                    "package `{}` not found in workspace `{}`",
                    name,
                    workspace.display()
                )
            }
            Self::ManifestNotInWorkspace {
                manifest,
                workspace_manifest,
            } => {
                return write!(f, "current package believes it's in a workspace when it's not:
current:   {}
workspace: {workspace_manifest_path}

this may be fixable by adding `{package_subpath}` to the `workspace.members` array of the manifest located at: {workspace_manifest_path}
Alternatively, to keep it out of the workspace, add an empty `[workspace]` table to the package's manifest.",
                    // TODO: Parse workspace.exclude and add back "add the package to the `workspace.exclude` array, or"
                    manifest.display(),
                    package_subpath = manifest.parent().unwrap().strip_prefix(workspace_manifest.parent().unwrap()).unwrap().display(),
                    workspace_manifest_path = workspace_manifest.display(),
                )
            },
            Self::Io(path, error) => return write!(f, "{}: {}", path.display(), error),
            Self::Toml(file, error) => return write!(f, "{}: {}", file.display(), error),
            Self::BinNotFound(name) => return write!(f, "Can't find `{name}` bin at `src/bin/{name}.rs` or `src/bin/{name}/main.rs`. Please specify bin.path if you want to use a non-default path.", name = name),
            Self::ExampleNotFound(name) => return write!(f, "Can't find `{name}` example at `examples/{name}.rs` or `examples/{name}/main.rs`. Please specify examples.path if you want to use a non-default path.", name = name),
            Self::DuplicateBin(name) => return write!(f, "found duplicate binary name {name}, but all binary targets must have a unique name"),
            Self::DuplicateExample(name) => return write!(f, "found duplicate example name {name}, but all example targets must have a unique name"),
        })
    }
}

impl std::error::Error for Error {}

impl From<PatternError> for Error {
    fn from(error: PatternError) -> Self {
        Self::GlobPatternError(error.msg)
    }
}

impl From<GlobError> for Error {
    fn from(error: GlobError) -> Self {
        Self::Glob(error)
    }
}
