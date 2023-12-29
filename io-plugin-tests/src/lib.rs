use io_plugin::io_plugin;
use rmp_serde::{from_read, to_vec};
use std::error::Error;
#[allow(unused_imports)]
use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
};

#[io_plugin()]
enum ExamplePlugin {
    GetName(String),
    UpdateConfig(String, HashMap<String, String>, (bool, u32)),
}

struct Handle {}
impl ExamplePluginHandle for Handle {
    async fn message(
        &mut self,
        _message: ExamplePluginMessage,
    ) -> Result<ExamplePluginResponse, Box<dyn Error>> {
        todo!()
    }
}

struct Plugin {}

impl ExamplePluginTrait for Plugin {
    fn get_name(&mut self) -> String {
        "Example".to_string()
    }

    fn update_config(&mut self, _instance: String, _arg2: HashMap<String, String>) -> (bool, u32) {
        (true, 1)
    }
}
