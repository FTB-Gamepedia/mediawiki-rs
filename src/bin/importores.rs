extern crate mediawiki;
use mediawiki::{Mediawiki, oredict::Oredict};
use std::fs::File;
use std::io::{BufWriter, Write};

fn import() {
    let path = "All Ores.txt";
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let mut file = BufWriter::new(File::create(path).unwrap());
    for ore in mw.query_ores() {
        let ore = ore.unwrap();
        let ore = ore.as_object().unwrap();
        let tag_name = ore["tag_name"].as_str().unwrap();
        let mod_name = ore["mod_name"].as_str().unwrap();
        let item_name = ore["item_name"].as_str().unwrap();
        let grid_params = ore["grid_params"].as_str().unwrap();
        let id = ore["id"].as_str().unwrap();
        if mod_name != "V" { continue }
        // if grid_params == "" { continue }
        // writeln!(&mut file, "{} {} = '{}' from {} ({})", id, tag_name, item_name, mod_name, grid_params).unwrap();
        writeln!(&mut file, "{}!{}!{}!{}", tag_name, item_name, mod_name, grid_params).unwrap();
    }
}
fn main() {
    import();
}
