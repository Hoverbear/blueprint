mod handlebars;
mod copy;


use toml::{Table, Array, Value};
use std::collections::BTreeMap;
use Task;

use error::{Error, Result};

pub use worker::handlebars::Handlebars;
pub use worker::copy::Copy;

pub trait Worker {
    fn new(config: BTreeMap<String, Value>) -> Result<Self> where Self: Sized;
    fn run(&self, task: &Task) -> Result<()>;
}
