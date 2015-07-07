use toml::{Table, Array, Value};
use worker::Worker;
use std::collections::BTreeMap;
use error::{Error, Result, BlueprintError};
use worker;

pub type Task = BTreeMap<String, Value>;

pub struct Blueprint {
    workers: BTreeMap<String, Box<Worker>>,
}

impl Blueprint {
    pub fn new(mut config: BTreeMap<String, Value>) -> Result<Self> {
        let workers = try!(config.get("Worker").and_then(|val| {
            match val {
                &Value::Table(ref t) => Some(t.clone()),
                _ => None,
            }
        }).ok_or(BlueprintError::Config("No `Worker` found in config.")));

        Ok(Blueprint {
            workers: Self::load_workers(workers).unwrap(),
        })
    }

    fn load_workers(workers: BTreeMap<String, Value>) -> Result<BTreeMap<String, Box<Worker>>> {
        workers.into_iter().map(|(key, val)| {
            let options = match val {
                Value::Table(t)  => t,
                _                => return Err(BlueprintError::Config("Worker is not table").into()),
            };
            if key == "Handlebars" {
                worker::Handlebars::new(options).map(|v| (key, Box::new(v) as Box<Worker>))
            } else if key == "Copy" {
                worker::Copy::new(options).map(|v| (key, Box::new(v) as Box<Worker>))
            } else {
                Err(BlueprintError::Config("Worker is not known").into())
            }
        }).collect()
    }

    pub fn run_task(&mut self, name: &String, task: &Task) -> Result<()> {
        println!("DEBUG {:?}", self.workers.keys().collect::<Vec<_>>());
        task.get("worker")
            .ok_or(BlueprintError::Config("Worker not specified.").into())
            .and_then(|worker_string| {
                match worker_string {
                    &Value::String(ref val) => self.workers.get(val)
                                                .ok_or(BlueprintError::Config("Worker not found.").into()),
                    _                   => Err(BlueprintError::Config("Worker should be string.").into()),
                }
            }).and_then(|worker| {
                println!("Running {:?}", name);
                worker.run(task)
            })
    }
}
