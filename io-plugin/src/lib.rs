use std::{error::Error, fmt::Display};

pub use io_plugin_macros::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IOPluginError(pub String);

impl Display for IOPluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for IOPluginError {}
