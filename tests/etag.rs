extern crate etag;

use etag::Etag;
use std::fs;
use std::time;

#[test]
fn calc_string() {
    let string = "ololo".to_string();
    let result = string.etag();

    assert!(result.starts_with("\"5"));
}

#[test]
fn calc_buff() {
    let buff = [1, 2, 3, 4];
    let result = buff.etag();

    assert!(result.starts_with("\"4"));
}

#[test]
fn calc_metadata() {
    let file = fs::File::open(file!()).expect("Couldn't open file!");
    let metadata = file.metadata().expect("Couldn't get metadata!");
    let modified = metadata.modified().unwrap().duration_since(time::UNIX_EPOCH).expect("Modified is earlier than time::UNIX_EPOCH!");
    let result = metadata.etag();

    assert_eq!(result, format!("\"{}.{}-{}\"", modified.as_secs(), modified.subsec_nanos(), metadata.len()));
}
