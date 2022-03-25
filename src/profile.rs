use crate::error::Error;
use std::path::Path;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Profile {
    Dev,
    Release,
    Custom(String),
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Dev => "dev",
            Self::Release => "release",
            Self::Custom(custom) => custom,
        })
    }
}

impl std::str::FromStr for Profile {
    type Err = Error;

    fn from_str(profile: &str) -> Result<Self, Self::Err> {
        Ok(match profile {
            "dev" => Profile::Dev,
            "release" => Profile::Release,
            custom => Profile::Custom(custom.into()),
        })
    }
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
