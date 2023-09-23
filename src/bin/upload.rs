use mediawiki::{Mediawiki, Upload};
fn upload_test(filename: &str, file: Upload, comment: Option<&str>) {
    let mw = Mediawiki::login_path("ftb.json").unwrap();
    let token = mw.get_token().unwrap();
    let result = mw
        .upload(filename, &token, file, None, comment, false)
        .unwrap();
    println!("{result:?}");
    match result["upload"]["result"].as_str().unwrap() {
        "Warning" => (),
        "Success" => return,
        other => panic!("Unknown result: {other}"),
    }
    for (warning, _value) in result["upload"]["warnings"].as_object().unwrap() {
        match &**warning {
            "was-deleted" => (),
            "duplicate" => (),
            "exists" => (),
            other => panic!("Unknown warning: {other}"),
        }
    }
    let filekey = result["upload"]["filekey"].as_str().unwrap();
    let result = mw
        .upload(
            filename,
            &token,
            Upload::Filekey(filekey),
            None,
            comment,
            true,
        )
        .unwrap();
    println!("{result:?}");
    match result["upload"]["result"].as_str().unwrap() {
        "Warning" => (),
        "Success" => (),
        other => panic!("Unknown result: {other}"),
    }
}
fn main() {
    upload_test(
        "Test.png",
        Upload::File(r"Test.png".as_ref()),
        Some("This is a test upload"),
    );
}
