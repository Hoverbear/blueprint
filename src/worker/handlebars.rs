use handlebars;

use toml::{Table, Array, Value};
use std::collections::BTreeMap;
use error::{Error, BlueprintError, Result};
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::Read;
use std::convert::From;

use std::env::current_dir;

use worker::Worker;
use Task;

pub struct Handlebars {
    compiler: handlebars::Handlebars,
}

impl Worker for Handlebars {
    fn new(config: BTreeMap<String, Value>) -> Result<Self> where Self: Sized {
        let mut worker = Handlebars {
            compiler: handlebars::Handlebars::new(),
        };

        // Read in data of default.
        let path = config.get("templates").and_then(|val| {
            match val {
                &Value::String(ref string) => Some(string.clone()),
                 _ => None,
            }
        }).unwrap_or(String::from("./templates"));


        // Read in templates.
        fs::read_dir(&path)
            .map_err(|e| e.into())
            .and_then(|templates| -> Result<Self> {
                templates.map(|entry| -> Result<()> {
                    let path = try!(entry.map(|entry| entry.path()).into());
                    // Need the name for the template name.
                    let name = try!(path.file_name()
                        .and_then(|path| path.to_str())
                        .and_then(|string| {
                            let mut splitter = string.split(".");
                            splitter.next()
                        }).ok_or(BlueprintError::Worker("Could not get template name.")));
                    println!("Loading template {:?} from {:?}", name, path);
                    // The content of the file.
                    let content = try!(File::open(&path).map(|mut file| {
                            let mut string = String::new();
                            file.read_to_string(&mut string).unwrap();
                            string
                    }));
                    // Tell Handlebars about it.
                    worker.compiler.register_template_string(name, content).map_err(|e| e.into())
                }).collect::<Result<Vec<()>>>().map(move |_| worker)
            })
    }
    fn run(&self, task: &Task) -> Result<()> {
        Ok(())
    }
}
