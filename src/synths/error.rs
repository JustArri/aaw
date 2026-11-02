use clack_host::{entry::PluginEntryError, plugin::PluginInstanceError};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] anyhow::Error),

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

    #[error("could not load clap plugin '{}:{plugin_id}': {source}", plugin_path.display())]
    ClapPluginLoad { plugin_path: PathBuf, plugin_id: String, source: PluginEntryError },

    #[error("invalid clap plugin '{}:{plugin_id}'", plugin_path.display())]
    InvalidClapPlugin { plugin_path: PathBuf, plugin_id: String },

    #[error("missing clap plugin '{}:{plugin_id}'", plugin_path.display())]
    MissingClapPlugin { plugin_path: PathBuf, plugin_id: String },

    #[error("could not instantiate clap plugin '{}:{plugin_id}': {source}", plugin_path.display())]
    ClapPluginInstance { plugin_path: PathBuf, plugin_id: String, source: PluginInstanceError },

    #[error("clap plugin missing feature: instrument")]
    ClapPluginNotInstrument,

    #[error("invalid CLAP parameter {parameter_id}: {reason}")]
    InvalidClapParameter { parameter_id: u32, reason: &'static str },
}
