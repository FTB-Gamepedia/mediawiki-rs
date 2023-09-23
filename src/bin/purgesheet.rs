use mediawiki::{tilesheet::Tilesheet, Mediawiki};
use std::env::args;

fn purge(mod_name: &str) {
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let todelete: Vec<String> = mw
        .query_tiles(Some(mod_name))
        .into_iter()
        .map(|tile| {
            let tile = tile.unwrap();
            let tile = tile.as_object().unwrap();
            tile["id"].as_i64().unwrap().to_string()
        })
        .collect();
    let token = mw.get_token().unwrap();
    for chunk in todelete.chunks(100) {
        let blob = chunk.join("|");
        mw.delete_tiles(&token, &blob, Some("Purging tiles"))
            .unwrap();
    }
    mw.delete_sheet(&token, mod_name, Some("Purging tilesheet"))
        .unwrap();
}
fn main() {
    let name = args().nth(1).unwrap();
    purge(&name);
}
