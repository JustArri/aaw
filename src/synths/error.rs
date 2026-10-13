use std::fmt;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidArgument(&'static str),
    AssetIo { path: PathBuf, source: std::io::Error },
    AssetTask(tokio::task::JoinError),
    NonUtf8Path(PathBuf),
    PathContainsNul(PathBuf),
    Native(&'static str),
    InstrumentLoad(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) | Self::Native(message) => f.write_str(message),
            Self::AssetIo { path, source } => {
                write!(f, "could not read asset {}: {source}", path.display())
            }
            Self::AssetTask(source) => write!(f, "synth loading task failed: {source}"),
            Self::NonUtf8Path(path) => write!(f, "path is not valid UTF-8: {}", path.display()),
            Self::PathContainsNul(path) => {
                write!(f, "path contains a NUL byte: {}", path.display())
            }
            Self::InstrumentLoad(path) => {
                write!(f, "could not load instrument: {}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AssetIo { source, .. } => Some(source),
            Self::AssetTask(source) => Some(source),
            _ => None,
        }
    }
}
