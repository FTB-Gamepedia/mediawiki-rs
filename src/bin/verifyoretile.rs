extern crate mediawiki;
use mediawiki::{
    Mediawiki,
    oredict::Oredict,
    tilesheet::Tilesheet,
};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};

fn import() {
    let path = "Invalid Ores.txt";
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let mut file = BufWriter::new(File::create(path).unwrap());
    let tiles: HashSet<(String, String)> = mw.query_tiles(None).into_iter().map(|tile| {
        let tile = tile.unwrap();
        let tile = tile.as_object().unwrap();
        let name = tile["name"].as_str().unwrap();
        let modd = tile["mod"].as_str().unwrap();
        (name.into(), modd.into())
    }).collect();
    for ore in mw.query_ores() {
        let ore = ore.unwrap();
        let ore = ore.as_object().unwrap();
        let tag_name = ore["tag_name"].as_str().unwrap();
        let mod_name = ore["mod_name"].as_str().unwrap();
        let item_name = ore["item_name"].as_str().unwrap();
        let id = ore["id"].as_i64().unwrap();
        let pair = (item_name.into(), mod_name.into());
        if !tiles.contains(&pair) && (mod_name != "GT"/* || !tag_name.starts_with("ore")*/) && mod_name != "GT6-I" {
            writeln!(&mut file, "{} {} = {} ({})", id, tag_name, item_name, mod_name).unwrap();
        }
    }

}
fn main() {
    import();
}
