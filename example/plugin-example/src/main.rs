use io_plugin_example::{Error, ExamplePluginTrait};
use rand::{thread_rng, Rng};
use std::{error::Error as StdError, ops::Div};

struct Plugin {
    round: bool,
}

impl ExamplePluginTrait for Plugin {
    #[doc = r"Get the name of this plugin"]
    fn get_name(&mut self) -> Result<String, Box<dyn StdError>> {
        Ok("Division".to_string())
    }

    fn set_rounding(&mut self, new_rounding_value: bool) -> Result<(), Box<dyn StdError>> {
        self.round = new_rounding_value;
        Ok(())
    }

    fn float_op(&mut self, arg1: f64, arg2: f64) -> Result<f64, Box<dyn StdError>> {
        let intermediate = (arg1 as f64).div(arg2 as f64);
        if intermediate.is_nan() {
            Err(Error::MathError)?;
        }
        if self.round {
            Ok(intermediate.round())
        } else {
            Ok(intermediate)
        }
    }

    fn random_bytes(&mut self, amount: usize) -> Result<Vec<u8>, Box<dyn StdError>> {
        let mut vec = Vec::with_capacity(amount);
        vec.fill_with(|| thread_rng().gen());
        Ok(vec)
    }
}

fn main() {
    ExamplePluginTrait::main_loop(Plugin { round: true })
}
