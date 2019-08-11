use mediawiki::Mediawiki;
use std::{env::args, fs::File, io::Write};

fn import(name: &str) {
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let data = mw.download_file(name).unwrap().unwrap();
    let mut file = File::create(name).unwrap();
    file.write_all(&data).unwrap();
}

fn main() {
    let name = args().nth(1).unwrap();
    import(&name);
}
