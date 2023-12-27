use io_plugin::io_plugin;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigStatus {
    Great,
    Bad,
}

#[io_plugin(client_gate = "plugin", host_gate = "host")]
enum ExamplePlugin {
    GetName(String),
    UpdateConfig(String, HashMap<String, String>, bool),
    ConfigStatus(String, (ConfigStatus, u32)),
    Stuff(String, ()),
}