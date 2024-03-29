#![feature(trait_alias)]
mod protocol;
mod tokio_exports;
mod process;

pub use io_plugin_macros::*;
pub use tokio_exports::*;
pub use process::*;
pub use protocol::{io_read, io_read_async, io_write, io_write_async};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::{ChildStdin, ChildStdout};

#[derive(Debug, Serialize, Deserialize, Error)]
pub enum IOPluginError {
    #[error("Pipe has been closed")]
    PipeClosed,
    #[error("Plugin failed to initialise: {0}")]
    InitialisationError(String),
    #[error("{0}")]
    Other(String),
}

pub struct ChildStdio {
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
}

pub type Mutex<T> = tokio::sync::Mutex<T>;
pub type Child = tokio::process::Child;

pub trait Serialise = serde::Serialize;
pub trait Deserialise = serde::de::DeserializeOwned;

pub type GenericValue = serde_cbor::Value;
