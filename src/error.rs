use std::fmt::{Display, Formatter, Result};
use std::io::Error as IoError;
use toml::de::Error as TomlError;
use glob::{PatternError, GlobError};

#[derive(Debug)]
pub enum Error {
    InvalidArgs,
    ManifestNotFound,
    RustcNotFound,
    Io(IoError),
    GlobPatternError(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let msg = match self {
            Self::InvalidArgs => "Invalid args.",
            Self::ManifestNotFound => "Didn't find Cargo.toml.",
            Self::RustcNotFound => "Didn't find rustc.",
            Self::Io(error) => return error.fmt(f),
            Self::GlobPatternError(error) => error,
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for Error {}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}

impl From<TomlError> for Error {
    fn from(_error: TomlError) -> Self {
        Self::ManifestNotFound
    }
}

impl From<PatternError> for Error {
    fn from(error: PatternError) -> Self {
        Self::GlobPatternError(error.msg)
    }
}

impl From<GlobError> for Error {
    fn from(error: GlobError) -> Self {
        Self::Io(error.into_error())
    }
}
