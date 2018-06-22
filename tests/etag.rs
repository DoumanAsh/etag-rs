extern crate etag;

use std::fs;
use std::time;

use etag::EntityTag;

#[test]
fn test_from_file_meta() {
    let file = fs::File::open("Cargo.toml").expect("To open Cargo.toml");
    let metadata = file.metadata().expect("To get metadata");
    let etag = EntityTag::from_file_meta(&metadata);

    assert_eq!(etag.weak, true);
    //Make sure we stick to format
    match metadata.modified().map(|modified| modified.duration_since(time::UNIX_EPOCH).expect("Modified is earlier than time::UNIX_EPOCH!")) {
        Ok(modified) => assert_eq!(format!("{}.{}-{}", modified.as_secs(), modified.subsec_nanos(), metadata.len()), etag.tag()),
        _ => assert_eq!(format!("{}", metadata.len()), etag.tag())
    }
}

#[test]
fn test_cmp() {
    const FIRST: &'static str = "FIRST";
    const SECOND: &'static str = "SECOND";

    let etag1 = EntityTag::weak(FIRST.to_owned());
    let etag2 = EntityTag::weak(FIRST.to_owned());
    assert!(!etag1.strong_eq(&etag2));
    assert!(etag1.weak_eq(&etag2));
    assert!(etag1.strong_ne(&etag2));
    assert!(!etag1.weak_ne(&etag2));

    let etag1 = EntityTag::weak(FIRST.to_owned());
    let etag2 = EntityTag::weak(SECOND.to_owned());
    assert!(!etag1.strong_eq(&etag2));
    assert!(!etag1.weak_eq(&etag2));
    assert!(etag1.strong_ne(&etag2));
    assert!(etag1.weak_ne(&etag2));

    let etag1 = EntityTag::weak(FIRST.to_owned());
    let etag2 = EntityTag::strong(FIRST.to_owned());
    assert!(!etag1.strong_eq(&etag2));
    assert!(etag1.weak_eq(&etag2));
    assert!(etag1.strong_ne(&etag2));
    assert!(!etag1.weak_ne(&etag2));

    let etag1 = EntityTag::strong(FIRST.to_owned());
    let etag2 = EntityTag::strong(FIRST.to_owned());
    assert!(etag1.strong_eq(&etag2));
    assert!(etag1.weak_eq(&etag2));
    assert!(!etag1.strong_ne(&etag2));
    assert!(!etag1.weak_ne(&etag2));
}

#[test]
fn test_etag_fmt() {
    assert_eq!(format!("{}", EntityTag::strong("foobar".to_owned())), "\"foobar\"");
    assert_eq!(format!("{}", EntityTag::strong("".to_owned())), "\"\"");
    assert_eq!(format!("{}", EntityTag::weak("weak-etag".to_owned())), "W/\"weak-etag\"");
    assert_eq!(format!("{}", EntityTag::weak("\u{0065}".to_owned())), "W/\"\x65\"");
    assert_eq!(format!("{}", EntityTag::weak("".to_owned())), "W/\"\"");
}

#[test]
fn test_etag_parse_success() {
    assert_eq!("\"foobar\"".parse::<EntityTag>().unwrap(), EntityTag::strong("foobar".to_owned()));
    assert_eq!("\"\"".parse::<EntityTag>().unwrap(), EntityTag::strong("".to_owned()));
    assert_eq!("W/\"weaktag\"".parse::<EntityTag>().unwrap(), EntityTag::weak("weaktag".to_owned()));
    assert_eq!("W/\"\x65\x62\"".parse::<EntityTag>().unwrap(), EntityTag::weak("\x65\x62".to_owned()));
    assert_eq!("W/\"\"".parse::<EntityTag>().unwrap(), EntityTag::weak("".to_owned()));
}

#[test]
fn test_etag_parse_failures() {
    assert!("W/\"ろり\"".parse::<EntityTag>().is_err());
    assert!("no-dquotes".parse::<EntityTag>().is_err());
    assert!("w/\"the-first-w-is-case-sensitive\"" .parse::<EntityTag>() .is_err());
    assert!("".parse::<EntityTag>().is_err());
    assert!("\"unmatched-dquotes1".parse::<EntityTag>().is_err());
    assert!("unmatched-dquotes2\"".parse::<EntityTag>().is_err());
    assert!("matched-\"dquotes\"".parse::<EntityTag>().is_err());
}
