// Copyright © 2016, Peter Atashian
extern crate mediawiki;
extern crate rustc_serialize;
use mediawiki::{Mediawiki};
use rustc_serialize::json::{decode};
use std::fs::{File};
use std::io::{Read};

fn main() {
    let mut file = File::open("ftb.json").unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let config = decode(&s).unwrap();
    let mw = Mediawiki::login(config).unwrap();
    println!("{:?}", mw.get_token::<mediawiki::Csrf>());
    println!("{:?}", mw.get_token::<mediawiki::Watch>());
    println!("{:?}", mw.get_token::<mediawiki::Patrol>());
    println!("{:?}", mw.get_token::<mediawiki::Rollback>());
    println!("{:?}", mw.get_token::<mediawiki::UserRights>());
}