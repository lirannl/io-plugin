use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin file {0} failed to start")]
    ExecutionError(PathBuf),
    #[error("Plugin ")]
    InitialisationError,

}
