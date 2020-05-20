use std::fmt::Display;

#[derive(Debug)]
/// Custom Error enum to ease error handling.
pub enum Error {
    /// For errors related to file processing
    FileError(std::io::Error),
    /// For errors related to YAML parsing
    YamlError(serde_yaml::Error),
    /// For errors related to server verification
    VerificationError(&'static str),
    /// For errors related to the server subprocess error
    SubprocessError(&'static str),
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
