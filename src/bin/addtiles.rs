// Copyright © 2016, Peter Atashian
extern crate mediawiki;
extern crate regex;
use mediawiki::{Csrf, Mediawiki};
use regex::{Regex};
use std::env::args;
use std::fs::{File};
use std::io::{Read};

fn addtiles(abbr: &str) {
    let path = "Added.txt";
    let mw = Mediawiki::login_file("ftb.json").unwrap();
    let mut file = File::open(path).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let reg = Regex::new(r"(\d+) (\d+) (.+)").unwrap();
    let token = mw.get_token::<Csrf>().unwrap();
    println!("{:?}", mw.create_sheet(&token, abbr, "16|32").unwrap());
    let tiles = s.lines().map(|line| {
        let cap = reg.captures(line).unwrap();
        let x: u32 = cap[1].parse().unwrap();
        let y: u32 = cap[2].parse().unwrap();
        let name = &cap[3];
        format!("{} {} {}", x, y, name)
    }).collect::<Vec<_>>();
    for chunk in tiles.chunks(100) {
        let chunk = chunk.join("|");
        println!("{:?}", mw.add_tiles(&token, abbr, &chunk));
    }
}

fn main() {
    let abbr = args().nth(1).unwrap();
    addtiles(&abbr);
}
