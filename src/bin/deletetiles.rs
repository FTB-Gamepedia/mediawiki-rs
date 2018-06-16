// Copyright © 2016, Peter Atashian
extern crate mediawiki;
use mediawiki::{Csrf, Mediawiki};
use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Read;

fn deletetiles(abbr: &str) {
    let path = "Deleted.txt";
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let mut file = File::open(path).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    /*
    let map: HashMap<String, u64> = mw.query_tiles(Some(abbr)).map(|tile| {
        let tile = tile.unwrap();
        let tile = tile.as_object().unwrap();
        let name = tile["name"].as_str().unwrap();
        let id = tile["id"].as_u64().unwrap();
        (name.into(), id)
    }).collect();
    let tiles: Vec<String> = s.lines().map(|line| map[line].to_string()).collect();
    let token = mw.get_token::<Csrf>().unwrap();
    for chunk in tiles.chunks(100) {
        let chunk = chunk.join("|");
        // println!("{:?}", chunk);
        println!("{:?}", mw.delete_tiles(&token, &chunk));
    }
    */
}

fn main() {
    let abbr = args().nth(1).unwrap();
    deletetiles(&abbr);
}
