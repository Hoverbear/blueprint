
extern crate handlebars;
extern crate hoedown;
extern crate rustc_serialize;
#[macro_use] extern crate itertools;
extern crate toml;
extern crate docopt;

mod error;
use error::{Result, Error};
mod worker;
mod blueprint;
use blueprint::Blueprint;

use toml::{Table, Array, Value};
use rustc_serialize::json::{Json, ToJson};
use rustc_serialize::Decodable;
use std::collections::BTreeMap;
use itertools::Itertools;
use docopt::Docopt;

use std::io::{self, Read, Write};
use std::fs::{self, PathExt, DirEntry};
use std::path::Path;
use std::env::{set_current_dir, current_dir};
use std::path::PathBuf;
use std::fs::{File, create_dir_all};

static CONFIG: &'static str = "Blueprint.toml";
static USAGE: &'static str = "
Usage: blueprint
";

pub use blueprint::Task;

#[derive(RustcDecodable, Debug)]
struct Args;

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    // Open the config file and parse it.
    let mut config_path = PathBuf::from(CONFIG);
    println!("Reading config at {:?}", config_path);
    let mut config = File::open(config_path).map(|mut file| {
        let mut config_string = String::new();
        file.read_to_string(&mut config_string);
        config_string
    }).map(|string| {
        toml::Parser::new(&string).parse().expect("No configuration file found.")
    }).unwrap();

    let tasks =  config.get("Task").map(|value| {
        match value {
            &Value::Table(ref tasks) => tasks.iter().map(|(k,v)| {
                match v {
                    &Value::Table(ref table) => (k.clone(),table.clone()),
                    _ => panic!("Failure parsing tasks."),
                }
            }).collect::<BTreeMap<String, Table>>(),
            _ => panic!("Tasks not found."),
        }
    }).unwrap();

    let mut blueprint = Blueprint::new(config)
        .unwrap();

    tasks.iter()
    .inspect(|&(ref key, ref task)| println!("Starting {:?}: {:#?}", key, task))
    .map(|(key, task)| (key.clone(), blueprint.run_task(key, task)))
    .inspect(|&(ref key, ref result)| println!("Result: {:?}: {:#?}", key, result))
    .collect::<BTreeMap<String, Result<()>>>();

    // // Read in all the templates.
    // let mut templates_path = PathBuf::from(&source);
    // templates_path.push(&template_dir);
    // let templates = fs::read_dir(&templates_path)
    //     .unwrap();
    // for entry in templates {
    //     let path = entry.map(|entry| {
    //         entry.path()
    //     }).unwrap();
    //     // Need the name for the template name.
    //     let name = path.file_name().map(|path| {
    //         let string = path.to_str().unwrap();
    //         let mut splitter = string.split(".");
    //         splitter.next().unwrap()
    //     }).unwrap();
    //     // The content of the file.
    //     let content = File::open(&path).map(|mut file| {
    //             let mut string = String::new();
    //             file.read_to_string(&mut string).unwrap();
    //             string
    //     }).unwrap();
    //     // Tell Handlebars about it.
    //     println!("Loading template {:?} from {:?}", name, path);
    //     handlebars.register_template_string(name, content)
    //       .ok().unwrap();
    // }
    //
    // // TODO: Register helpers.
    //
    // // Build all the pages.
    // let mut pages_path = PathBuf::from(&source);
    // pages_path.push(page_dir);
    // let pages = fs::read_dir(pages_path)
    //     .unwrap();
    // let mut dest_path = PathBuf::from(&initial_dir);
    // dest_path.push(&dest);
    // create_dir_all(&dest_path).unwrap();
    //
    // for entry in pages {
    //     let path = entry.map(|entry| {
    //         entry.path()
    //     }).unwrap();
    //
    //     // Need the name for the template name.
    //     let name = path.file_name().map(|path| {
    //         let string = path.to_str().unwrap();
    //         let mut splitter = string.split(".");
    //         splitter.next().unwrap()
    //     }).unwrap();
    //
    //     // And the data.
    //     let data = File::open(&path).map(|mut file| {
    //             let mut string = String::new();
    //             file.read_to_string(&mut string).unwrap();
    //
    //             // Metadata is toml. Detect the split.
    //             let mut splitter = string.split("------");
    //             let meta_str = splitter.next()
    //                 .unwrap();
    //             let meta = toml::Parser::new(&meta_str).parse()
    //                 .expect("No metadata found.");
    //             let title = meta.get("title").map(|v| {
    //                 v.as_str().unwrap()
    //             }).unwrap_or("");
    //             let author = meta.get("author").map(|v| {
    //                 v.as_str().unwrap()
    //             }).unwrap_or("");
    //             let template = meta.get("template").map(|v| {
    //                 v.as_str().unwrap()
    //             }).unwrap_or("");
    //
    //             // Content is super easy to get.
    //             let content = splitter.next()
    //                 .expect("Could not detect metadata in file.");
    //             let content_md = Markdown::new(&content);
    //             let content_html = html.render(&content_md);
    //
    //             // Build the page.
    //             Page {
    //                 title: title.into(),
    //                 author: author.into(),
    //                 template: template.into(),
    //                 content:  content_html.to_str().unwrap().into(),
    //             }
    //     }).unwrap();
    //
    //     // Spout the result to a file.
    //     let out_html = handlebars.render(&data.template, &data).unwrap();
    //     let mut out_path = PathBuf::from(&initial_dir);
    //     out_path.push(&dest);
    //     out_path.push(name);
    //     if name != "index" {
    //         create_dir_all(&out_path).unwrap();
    //         out_path.push("index");
    //     }
    //     out_path = out_path.with_extension("html");
    //     println!("Rendering {:?} to {:?}", path, out_path);
    //     File::create(&out_path).and_then(|mut file| {
    //         file.write(&out_html.as_bytes())
    //     }).unwrap();
    // }
    //
    // // Copy over stylesheets.
    // let mut style_path = PathBuf::from(&source);
    // style_path.push(style_dir);
    // let mut style_dest = PathBuf::from(&initial_dir);
    // style_dest.push(&dest);
    // style_dest.push("styles");
    // create_dir_all(&style_dest).unwrap();
    // let styles = fs::read_dir(style_path)
    //     .unwrap();
    // for entry in styles {
    //     let from = entry.map(|entry| {
    //         entry.path()
    //     }).unwrap();
    //     let mut to = style_dest.clone();
    //     to.push(&from.file_name().and_then(|v| v.to_str()).unwrap());
    //     println!("Copying {:?} to {:?}", from, to);
    //     fs::copy(from, to).unwrap();
    // }
}
