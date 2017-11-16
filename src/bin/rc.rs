// Copyright © 2016, Peter Atashian
extern crate mediawiki;
use mediawiki::{Error, Mediawiki};
use std::collections::{HashMap};
use std::fs::{create_dir, File};
use std::io::{Write};
use std::path::{Path, PathBuf};

fn main() {
    let base = Path::new("rc");
    let mut files: HashMap<PathBuf, File> = HashMap::new();
    let mw = Mediawiki::login_file("ftb.json").unwrap();
    for change in mw.query_recentchanges(5000) {
        if let Err(e) = (|| -> Result<(), Error> {
            let change = change?;
            let kind = change.get("type")?.as_str()?;
            let name = if kind == "log" {
                let logtype = change.get("logtype")?.as_str()?;
                let logaction = change.get("logaction")?.as_str()?;
                let _ = create_dir(base.join(logtype));
                base.join(logtype).join(logaction).with_extension("json")
            } else {
                base.join(kind).with_extension("json")
            };
            let mut file = files.entry(name.clone())
                .or_insert_with(|| File::create(name).unwrap());
            writeln!(&mut file, "{}", change)?;
            Ok(())
        })() {
            println!("Error: {:?}", e);
        }
    }
}
