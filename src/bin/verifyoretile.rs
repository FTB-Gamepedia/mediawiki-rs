use mediawiki::{oredict::Oredict, tilesheet::Tilesheet, Mediawiki};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufWriter, Write},
};

fn import() {
    let path = "Invalid Ores.txt";
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let mut file = BufWriter::new(File::create(path).unwrap());
    let tiles: HashSet<(String, String)> = mw
        .query_tiles(None)
        .into_iter()
        .map(|tile| {
            let tile = tile.unwrap();
            let tile = tile.as_object().unwrap();
            let name = tile["name"].as_str().unwrap();
            let modd = tile["mod"].as_str().unwrap();
            (name.into(), modd.into())
        })
        .collect();
    let mut todelete: Vec<String> = Vec::new();
    for ore in mw.query_ores() {
        let ore = ore.unwrap();
        let ore = ore.as_object().unwrap();
        let tag_name = ore["tag_name"].as_str().unwrap();
        let mod_name = ore["mod_name"].as_str().unwrap();
        let item_name = ore["item_name"].as_str().unwrap();
        let id = ore["id"].as_i64().unwrap();
        let pair = (item_name.into(), mod_name.into());
        //if !tiles.contains(&pair) {
        if !tiles.contains(&pair) {
            if mod_name == "GT6-I" {
                todelete.push(id.to_string());
            }
            writeln!(
                &mut file,
                "{} {} = {} ({})",
                id, tag_name, item_name, mod_name
            )
            .unwrap();
        }
    }
    /*
    let token = mw.get_token().unwrap();
    for chunk in todelete.chunks(100) {
        let blob = chunk.join("|");
        mw.delete_ores(&token, &blob).unwrap();
    }*/
}
fn main() {
    import();
}
