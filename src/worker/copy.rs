use worker::Worker;

use toml::{Table, Array, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::{self, File, create_dir_all};
use std::io::{Read, Write};
use error::{Error, Result, BlueprintError};


use Task;
pub struct Copy;

impl Worker for Copy {
    fn new(config: BTreeMap<String, Value>) -> Result<Self> where Self: Sized {
        Ok(Copy)
    }
    fn run(&self, task: &Task) -> Result<()> {
        // Read in data of default.
        let source = task.get("source").and_then(|val| {
            match val {
                &Value::String(ref string) => Some(string.clone()),
                 _ => None,
            }
        }).map(|val| {
            PathBuf::from(val)
        }).unwrap_or(PathBuf::from("./pages"));

        let dest = task.get("dest").and_then(|val| {
            match val {
                &Value::String(ref string) => Some(string.clone()),
                 _ => None,
            }
        }).map(|val| {
            PathBuf::from(val)
        }).unwrap_or(PathBuf::from("./out"));

        fs::read_dir(&source)
        .map_err(|e| e.into())
        .and_then(|pages| {
            pages.map(|entry| {
                let path = try!(entry.map(|entry| entry.path()).into());
                let name = try!(path.file_name()
                    .and_then(|path| path.to_str())
                    .ok_or(BlueprintError::Worker("Could not get page name.")));

                // The content of the file.
                let string = try!(File::open(&path).map(|mut file| {
                        let mut string = String::new();
                        file.read_to_string(&mut string).unwrap();
                        string
                }));

                // Create Pretty directory.
                let out_folder = dest.clone();
                create_dir_all(&out_folder).unwrap();

                let out_path = out_folder.join(name);
                File::create(&out_path).and_then(|mut file| {
                    file.write(&string.as_bytes())
                }).map(|_| ()).map_err(|e| e.into())
            }).collect::<Result<Vec<()>>>().map(|_| ())
        })
    }
}
