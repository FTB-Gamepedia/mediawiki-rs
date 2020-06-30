use mediawiki::Mediawiki;
use std::path::Path;

fn main() {
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let token = mw.get_token().unwrap();
    let result = mw.upload_file("Test.png", Path::new(r"Test.png"), &token);
    println!("{:?}", result);
}
