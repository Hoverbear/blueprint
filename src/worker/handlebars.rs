use handlebars;
use handlebars::JsonRender;

use toml::{Table, Array, Value, Parser};
use rustc_serialize::json::Json;
use std::collections::BTreeMap;
use error::{Error, BlueprintError, Result};
use std::path::PathBuf;
use std::fs::{self, File, create_dir_all};
use std::io::{Read, Write};
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

        worker.compiler.register_helper("markdown", Box::new(MarkdownHelper));

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
                    // The content of the file.
                    let content = try!(File::open(&path).map(|mut file| {
                            let mut string = String::new();
                            file.read_to_string(&mut string).unwrap();
                            string
                    }));
                    // Tell Handlebars about it.
                    println!("Loading template {:?} from {:?}", name, path);
                    worker.compiler.register_template_string(name, content).map_err(|e| e.into())
                }).collect::<Result<Vec<()>>>().map(move |_| worker)
            })
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
                // Need the name for the template name.
                let name = try!(path.file_name()
                    .and_then(|path| path.to_str())
                    .and_then(|string| {
                        let mut splitter = string.split(".");
                        splitter.next()
                    }).ok_or(BlueprintError::Worker("Could not get page name.")));

                // The content of the file.
                let string = try!(File::open(&path).map(|mut file| {
                        let mut string = String::new();
                        file.read_to_string(&mut string).unwrap();
                        string
                }));

                // Metadata is toml. Detect the split.
                let mut splitter = string.split("------");
                let meta_str = splitter.next()
                    .unwrap();
                let mut meta = Parser::new(&meta_str).parse()
                    .expect("No metadata found.");
                let content_str = splitter.next()
                    .unwrap();
                meta.insert(String::from("content"), Value::String(String::from(content_str)));

                let template = meta.get("template").and_then(|val| {
                    match val {
                        &Value::String(ref string) => Some(string.clone()),
                         _ => None,
                    }
                }).unwrap_or(String::from("layout"));
                let json = convert(Value::Table(meta));

                // Create Pretty directory.
                let mut out_path = dest.clone().join(name);
                if name != "index" {
                    create_dir_all(&out_path).unwrap();
                    out_path.push("index")
                }
                out_path.with_extension("html");

                // Tell Handlebars about it.
                let out_html = try!(self.compiler.render(&template, &json));

                File::create(&out_path).and_then(|mut file| {
                    file.write(&out_html.as_bytes())
                }).map(|_| ()).map_err(|e| e.into())
            }).collect::<Result<Vec<()>>>().map(|_| ())
        })
    }
}

// implement by a structure impls HelperDef
#[derive(Clone, Copy)]
struct MarkdownHelper;

impl handlebars::HelperDef for MarkdownHelper {
    fn call(&self, c: &handlebars::Context, h: &handlebars::Helper, _: &handlebars::Handlebars,
    rc: &mut handlebars::RenderContext) -> ::std::result::Result<(), handlebars::RenderError> {
        let param = h.params().get(0).unwrap();
        let value = c.navigate(rc.get_path(), param);

        try!(rc.writer.write("Helper: ".as_bytes()));
        try!(rc.writer.write(value.render().into_bytes().as_ref()));
        Ok(())
    }
}

fn convert(toml: Value) -> Json {
    match toml {
        Value::String(s) => Json::String(s),
        Value::Integer(i) => Json::I64(i),
        Value::Float(f) => Json::F64(f),
        Value::Boolean(b) => Json::Boolean(b),
        Value::Array(arr) => Json::Array(arr.into_iter().map(convert).collect()),
        Value::Table(table) => Json::Object(table.into_iter().map(|(k, v)| {
            (k, convert(v))
        }).collect()),
        Value::Datetime(dt) => Json::String(dt),
    }
}
