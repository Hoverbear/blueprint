use worker::Worker;

use toml::{Table, Array, Value};
use std::collections::BTreeMap;
use error::{Error, Result};


use Task;
pub struct Copy;

impl Worker for Copy {
    fn new(config: BTreeMap<String, Value>) -> Result<Self> where Self: Sized {
        Ok(Copy)
    }
    fn run(&self, task: &Task) -> Result<()> {
        Ok(())
    }
}
