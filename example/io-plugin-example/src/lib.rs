use io_plugin::{handle_doc, io_plugin};
use rmp_serde::{from_read, to_vec};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
};
use thiserror::Error;

#[io_plugin()]
#[handle_doc("async `ExamplePlugin` handle. Kills process on drop")]
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
    #[error("Plugin failed to initialise: {0}")]
    InitialisationError(String),
    #[error("{0}")]
    Generic(String),
}
