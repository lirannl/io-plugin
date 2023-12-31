use io_plugin::{handle_doc, io_plugin};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[io_plugin]
#[derive(Clone)]
#[handle_doc("async `ExamplePlugin` handle")]
pub enum ExamplePlugin {
    ///Get the name of this plugin
    GetName(String),
    SetRounding(bool, ()),
    FloatOp(f64, f64, f64),
    /// Get `usize` random bytes from the plugin - used to simulate large data transfer
    RandomBytes(usize, Vec<u8>),
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum Error {
    #[error("The result is mathematically invalid")]
    MathError,
    #[error("{0}")]
    Generic(String),
}
