// Copyright © 2016, Peter Atashian
extern crate mediawiki;
use mediawiki::{Mediawiki, tilesheet::Tilesheet};
use std::env::args;
use std::fs::{File};
use std::io::{Write};

fn import(abbr: &str) {
    let path = format!(r"Tilesheet {}.txt", abbr);
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let mut file = File::create(path).unwrap();
    for tile in mw.query_tiles(Some(abbr)) {
        let tile = tile.unwrap();
        let tile = tile.as_object().unwrap();
        let x = tile["x"].as_u64().unwrap();
        let y = tile["y"].as_u64().unwrap();
        let name = tile["name"].as_str().unwrap();
        writeln!(&mut file, "{} {} {}", x, y, name).unwrap();
    }
}

fn main() {
    let abbr = args().nth(1).unwrap();
    import(&abbr);
}
