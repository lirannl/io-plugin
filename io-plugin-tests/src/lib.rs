use io_plugin::{io_plugin, plugin_trait_doc, handle_doc};
use rmp_serde::{from_read, to_vec};
use std::error::Error;
#[allow(unused_imports)]
use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
};

#[io_plugin()]
#[handle_doc("This is a handle")]
enum ExamplePlugin {
    ///Get the name of this plugin
    GetName(&'static str),
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
    fn get_name(&mut self) -> &'static str {
        "Example"
    }

    fn update_config(&mut self, _instance: String, _arg2: HashMap<String, String>) -> (bool, u32) {
        (true, 1)
    }
}
