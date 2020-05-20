use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    FileError(std::io::Error),
    YamlError(serde_yaml::Error),
    VerificationError(&'static str),
    ServerSpawnError(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // TODO actually format this correctly
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::FileError(e)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::YamlError(e)
    }
}
