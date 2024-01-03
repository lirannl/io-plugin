use io_plugin::{handle_doc, io_plugin};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature="plugin")]
use rand::{thread_rng, Rng};
#[cfg(feature="plugin")]
use std::error::Error as StdError;
use thiserror::Error;

#[io_plugin(plugin_trait = "plugin", handle = "host")]
#[derive(Clone)]
#[handle_doc("async `ExamplePlugin` handle")]
pub enum ExamplePlugin<T: DeserializeOwned + Serialize> {
    ///Get the name of this plugin
    GetName(String),
    SetState(i32, ()),
    GetState(i32),
    Op(f64, f64, T),
    ///Get `usize` random bytes from the plugin - used to simulate large data transfer
    #[implementation(gen_bytes)]
    RandomBytes(usize, Vec<u8>),
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum Error {
    #[error("The result is mathematically invalid")]
    MathError,
    #[error("{0}")]
    Generic(String),
}

#[cfg(feature="plugin")]
async fn gen_bytes<T: DeserializeOwned + Serialize>(
    _plugin: &mut dyn ExamplePluginTrait<T>,
    amount: usize,
) -> Result<Vec<u8>, Box<dyn StdError>> {
    let mut vec = Vec::with_capacity(amount);
    for _ in 0..amount {
        vec.push(thread_rng().gen())
    }
    Ok(vec)
}
