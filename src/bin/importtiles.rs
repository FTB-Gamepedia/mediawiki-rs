extern crate mediawiki;
use mediawiki::Mediawiki;
use std::fs::File;
use std::io::{BufWriter, Write};

fn import() {
    let path = "All Tiles.txt";
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let mut file = BufWriter::new(File::create(path).unwrap());
    for tile in mw.query_tiles(None) {
        let tile = tile.unwrap();
        let tile = tile.as_object().unwrap();
        let id = tile["id"].as_u64().unwrap();
        let x = tile["x"].as_u64().unwrap();
        let y = tile["y"].as_u64().unwrap();
        let name = tile["name"].as_str().unwrap();
        let mod_ = tile["mod"].as_str().unwrap();
        writeln!(&mut file, "{} {} {} {} {}", id, mod_, x, y, name).unwrap();
    }
}
fn main() {
    import();
}
