use io_plugin_example::{Error, ExamplePluginTrait};
use std::{error::Error as StdError, ops::Div};
use tokio::main;

struct Plugin {
    state: i32,
}

impl ExamplePluginTrait<f64> for Plugin {
    #[doc = r"Get the name of this plugin"]
    async fn get_name(&mut self) -> Result<String, Box<dyn StdError>> {
        Ok("Division".to_string())
    }

    async fn op(&mut self, arg1: f64, arg2: f64) -> Result<f64, Box<dyn StdError>> {
        let intermediate = (arg1 as f64).div(arg2 as f64);
        if intermediate.is_nan() {
            Err(Error::MathError)?;
        }
        Ok(intermediate)
    }

    async fn set_state(
        &mut self,
        new_state: i32,
    ) -> Result<(), Box<dyn StdError>>
    where
        Self: Sized,
    {
        self.state = new_state;
        Ok(())
    }

    async fn get_state(
        &mut self,
    ) ->Result<i32, Box<dyn StdError>>
    where
        Self: Sized,
    {
        Ok(self.state)
    }
}

#[main]
async fn main() {
    Plugin { state: 0 }.main_loop().await
}
