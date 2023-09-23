use mediawiki::{Error, Mediawiki};
use std::collections::HashMap;
use std::fs::{create_dir, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() {
    let base = Path::new("rc");
    let _ = create_dir(base);
    let mut files: HashMap<PathBuf, BufWriter<File>> = HashMap::new();
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    for change in mw.query_recentchanges(5000) {
        if let Err(e) = (|| -> Result<(), Error> {
            let change = change?;
            let kind = change["type"]
                .as_str()
                .ok_or_else(|| Error::Json(change.clone()))?;
            let name = if kind == "log" {
                let logtype = change["logtype"]
                    .as_str()
                    .ok_or_else(|| Error::Json(change.clone()))?;
                let logaction = change["logaction"]
                    .as_str()
                    .ok_or_else(|| Error::Json(change.clone()))?;
                let _ = create_dir(base.join(logtype));
                base.join(logtype).join(logaction).with_extension("json")
            } else {
                base.join(kind).with_extension("json")
            };
            let mut file = files
                .entry(name.clone())
                .or_insert_with(|| BufWriter::new(File::create(name).unwrap()));
            writeln!(&mut file, "{change}")?;
            Ok(())
        })() {
            println!("Error: {e:?}");
        }
    }
}
