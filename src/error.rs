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
    MultiplePackagesNotSupported,
    GlobPatternError(&'static str),
    Glob(GlobError),
    UnexpectedWorkspace(PathBuf),
    Io(PathBuf, IoError),
    Toml(PathBuf, TomlError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(match self {
            Self::InvalidArgs => "Invalid args.",
            Self::ManifestNotAWorkspace => "The provided Cargo.toml does not contain a `[workspace]`",
            Self::ManifestNotFound => "Didn't find Cargo.toml.",
            Self::ManifestPathNotFound => "The manifest-path must be a path to a Cargo.toml file",
            Self::MultiplePackagesNotSupported => {
                "Multiple packages are not yet supported (ie. in a `[workspace]`). Use `-p` to select a specific package."
            }
            Self::RustcNotFound => "Didn't find rustc.",
            Self::GlobPatternError(error) => error,
            Self::Glob(error) => return error.fmt(f),
            Self::UnexpectedWorkspace(path) => return write!(f, "Did not expect a `[workspace]` at {}", path.display()),
            Self::Io(path, error) => return write!(f, "{}: {}", path.display(), error),
            Self::Toml(file, error) => return write!(f, "{}: {}", file.display(), error),
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
