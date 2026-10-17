use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    InvalidArgument(&'static str),

    #[error("could not resolve asset path '{}': {source}", relative_path.display())]
    AssetPathResolution { relative_path: PathBuf, source: std::io::Error },

    #[error("could not read asset {}: {source}", path.display())]
    AssetIo { path: PathBuf, source: std::io::Error },

    #[error("synth loading task failed: {0}")]
    AssetTask(#[from] tokio::task::JoinError),

    #[error("path is not valid UTF-8: {}", .0.display())]
    NonUtf8Path(PathBuf),

    #[error("path contains a NUL byte: {}", .0.display())]
    PathContainsNul(PathBuf),

    #[error("{0}")]
    Native(&'static str),

    #[error("could not load instrument: {}", .0.display())]
    InstrumentLoad(PathBuf),
}
