use std::path::PathBuf;

use error::PluginError;
use futures::Future;
use serde::{Deserialize, Serialize};

pub mod error;
mod test;

pub use derive_io_plugin::IoPlugin;

pub trait IoPlugin<
    MessageType: Serialize + for<'a> Deserialize<'a>,
    ResponseType: Serialize + for<'a> Deserialize<'a>,
>
{
    fn init(
        message: MessageType,
        plugin_file: PathBuf,
    ) -> Box<dyn Future<Output = Result<Self, PluginError>>>
    where
        Self: Sized;
    fn message(message: MessageType)
        -> Box<dyn Future<Output = Result<ResponseType, PluginError>>>;
}
