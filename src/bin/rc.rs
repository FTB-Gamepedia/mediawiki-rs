// Copyright © 2016, Peter Atashian
extern crate mediawiki;
extern crate rustc_serialize;
use mediawiki::{JsonFun, Mediawiki};
use rustc_serialize::json::{decode};
use std::collections::{HashMap};
use std::fs::{create_dir, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

fn main() {
    let base = Path::new("rc");
    let mut files: HashMap<PathBuf, File> = HashMap::new();
    let mut config = File::open("ftb.json").unwrap();
    let mut s = String::new();
    config.read_to_string(&mut s).unwrap();
    let config = decode(&s).unwrap();
    let mw = Mediawiki::login(config).unwrap();
    for change in mw.query_recentchanges(5000) {
        let change = change.unwrap();
        let kind = change.get("type").string().unwrap_or("unknown");
        let name = if kind == "log" {
            let logtype = change.get("logtype").string().unwrap_or("unknown");
            let logaction = change.get("logaction").string().unwrap_or("unknown");
            let _ = create_dir(base.join(logtype));
            base.join(logtype).join(logaction).with_extension("json")
        } else {
            base.join(kind).with_extension("json")
        };
        let mut file = files.entry(name.clone()).or_insert_with(|| File::create(name).unwrap());
        writeln!(&mut file, "{}", change).unwrap();
    }
}
