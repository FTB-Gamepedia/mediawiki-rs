use mediawiki::{oredict::Oredict, tilesheet::Tilesheet, Mediawiki};
use serde_json::Value;
use std::fs;

fn export() {
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let tiles = mw
        .query_tiles(None)
        .into_iter()
        .map(|tile| tile.unwrap())
        .collect::<Vec<_>>();
    fs::write("tiles.json", Value::Array(tiles.clone()).to_string()).unwrap();
    let sheets = mw
        .query_sheets()
        .into_iter()
        .map(|sheet| sheet.unwrap())
        .collect::<Vec<_>>();
    fs::write("sheets.json", Value::Array(sheets).to_string()).unwrap();
    let ores = mw
        .query_ores(None)
        .into_iter()
        .map(|ore| ore.unwrap())
        .collect::<Vec<_>>();
    fs::write("ores.json", Value::Array(ores).to_string()).unwrap();
    let translations = tiles
        .into_iter()
        .flat_map(|tile| {
            let id = tile["id"].as_i64().unwrap();
            mw.query_tile_translations(id)
                .into_iter()
                .map(|trans| trans.unwrap())
        })
        .collect::<Vec<_>>();
    fs::write("translations.json", Value::Array(translations).to_string()).unwrap();
}
fn main() {
    export();
}
