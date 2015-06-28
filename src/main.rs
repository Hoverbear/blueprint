
extern crate handlebars;
extern crate hoedown;
extern crate rustc_serialize;
#[macro_use] extern crate itertools;
extern crate toml;
extern crate docopt;

use toml::{Table, Array};
use handlebars::Handlebars;
use hoedown::{Html, Markdown};
use hoedown::renderer::{Render, html};
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

static CONFIG: &'static str = "./Blueprint.toml";
static USAGE: &'static str = "
Usage: blueprint <source> <dest>
       blueprint <dest>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_source: String,
    arg_dest: String,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct PageData {
    title: String,
    author: String,
    content: String,
}
impl ToJson for PageData {
  fn to_json(&self) -> Json {
    let mut data: BTreeMap<String, Json> = BTreeMap::new();
    data.insert("title".to_string(), self.title.to_json());
    data.insert("author".to_string(), self.author.to_json());
    data.insert("content".to_string(), self.content.to_json());
    data.to_json()
  }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    let mut handlebars = Handlebars::new();
    let mut html = Html::new(html::Flags::empty(), 0);
    // Set the current directory to the source for ease of use.
    let source = PathBuf::from(args.arg_source);
    let dest = PathBuf::from(args.arg_dest);

    let initial_dir = current_dir()
        .unwrap();
    set_current_dir(source)
        .unwrap();

    // Open the config file and parse it.
    let mut config = File::open(CONFIG).map(|mut file| {
        let mut config_string = String::new();
        file.read_to_string(&mut config_string);
        config_string
    }).map(|string| {
        toml::Parser::new(&string).parse().expect("No configuration file found.")
    }).unwrap();

    // Get the configuration details.
    // let config = toml::Parser::new(&config_string).parse().unwrap();
    let style_dir = config.get("style_dir").map(|v| {
        v.as_str().expect("Could not parse style_dir")
    }).unwrap();
    let template_dir = config.get("template_dir").map(|v| {
        v.as_str().expect("Could not parse template_dir")
    }).unwrap();
    let page_dir = config.get("page_dir").map(|v| {
        v.as_str().expect("Could not parse page_dir")
    }).unwrap();

    // Read in all the templates.
    let templates = fs::read_dir(template_dir)
        .unwrap();
    for entry in templates {
        let path = entry.map(|entry| {
            entry.path()
        }).unwrap();
        // Need the name for the template name.
        let name = path.file_name().map(|path| {
            let string = path.to_str().unwrap();
            let mut splitter = string.split(".");
            splitter.next().unwrap()
        }).unwrap();
        // The content of the file.
        let mut content = File::open(&path).map(|mut file| {
                let mut string = String::new();
                file.read_to_string(&mut string).unwrap();
                string
        }).unwrap();
        // Tell Handlebars about it.
        handlebars.register_template_string(name, content)
          .ok().unwrap();
    }

    // TODO: Register helpers.

    // Build all the pages.
    let pages = fs::read_dir(page_dir)
        .unwrap();
    let mut dest_path = PathBuf::from(&initial_dir);
    dest_path.push(&dest);
    create_dir_all(&dest_path).unwrap();
    for entry in pages {
        let path = entry.map(|entry| {
            entry.path()
        }).unwrap();
        // Need the name for the template name.
        let name = path.file_name().map(|path| {
            let string = path.to_str().unwrap();
            let mut splitter = string.split(".");
            splitter.next().unwrap()
        }).unwrap();
        // And the data.
        let data = File::open(&path).map(|mut file| {
                let mut string = String::new();
                file.read_to_string(&mut string).unwrap();

                // Metadata is toml. Detect the split.
                let mut splitter = string.split("------");
                let meta_str = splitter.next()
                    .unwrap();
                let meta = toml::Parser::new(&meta_str).parse()
                    .expect("No metadata found.");
                let title = meta.get("title").map(|v| {
                    v.as_str().unwrap()
                }).unwrap_or("");
                let author = meta.get("author").map(|v| {
                    v.as_str().unwrap()
                }).unwrap_or("");

                // Content is super easy to get.
                let content = splitter.next()
                    .unwrap();
                let content_md = Markdown::new(&content);
                let content_html = html.render(&content_md);

                // Build the page.
                PageData {
                    title: title.into(),
                    author: author.into(),
                    content:  content_html.to_str().unwrap().into(),
                }
        }).unwrap();
        // Spout the result to a file.
        let out_html = handlebars.render("test", &data).unwrap();
        let mut out_path = PathBuf::from(&initial_dir);
        out_path.push(&dest);
        out_path.push(name);
        out_path = out_path.with_extension("html");
        println!("{:?}", out_path);
        File::create(&out_path).map(|mut file| {
            file.write(&out_html.as_bytes())
        }).unwrap();
    }
}
