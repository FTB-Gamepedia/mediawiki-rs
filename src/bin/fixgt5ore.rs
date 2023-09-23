use mediawiki::{oredict::Oredict, Mediawiki};
use std::collections::HashSet;

fn fix_gt5() {
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let token = mw.get_token().unwrap();
    let gt5_ores: HashSet<(String, String)> = mw
        .query_ores(Some("GT5"))
        .into_iter()
        .map(|ore| {
            let ore = ore.unwrap();
            let ore = ore.as_object().unwrap();
            let tag_name = ore["tag_name"].as_str().unwrap();
            let item_name = ore["item_name"].as_str().unwrap();
            (tag_name.into(), item_name.into())
        })
        .collect();
    for ore in mw.query_ores(Some("GT")) {
        let ore = ore.unwrap();
        let ore = ore.as_object().unwrap();
        let tag_name = ore["tag_name"].as_str().unwrap();
        let mod_name = ore["mod_name"].as_str().unwrap();
        let item_name = ore["item_name"].as_str().unwrap();
        // let grid_params = ore["grid_params"].as_str().unwrap();
        let id = ore["id"].as_i64().unwrap();
        assert!(mod_name == "GT");
        let pair = (tag_name.into(), item_name.into());
        if !gt5_ores.contains(&pair) {
            println!("{pair:?}");
            mw.edit_ore(&token, id, Some("GT5"), None, None, None)
                .unwrap();
        }
    }
}
fn main() {
    fix_gt5();
}
