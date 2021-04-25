extern crate etag;

use etag::EntityTag;

#[cfg(feature = "std")]
#[test]
fn test_from_file_meta() {
    use std::fs;

    let file = fs::File::open("Cargo.toml").expect("To open Cargo.toml");
    let metadata = file.metadata().expect("To get metadata");
    let etag = EntityTag::from_file_meta(&metadata);

    assert_eq!(etag.weak, true);
    //Make sure we stick to format
    match metadata.modified().map(|modified| modified.duration_since(std::time::UNIX_EPOCH).expect("Modified is earlier than time::UNIX_EPOCH!")) {
        Ok(modified) => assert_eq!(format!("{}.{}-{}", modified.as_secs(), modified.subsec_nanos(), metadata.len()), etag.tag()),
        _ => assert_eq!(format!("{}", metadata.len()), etag.tag())
    }
}

#[test]
fn test_etag_from_data() {
    const ZERO: &'static [u8] = b"";
    const FIRST: &'static [u8] = b"12";
    const SECOND: &'static [u8] = b"21";
    const BIG: &'static [u8] = b"123456789123456789123456789";
    const VERY_BIG: &'static [u8] = b"123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789123456789";

    let zero = EntityTag::from_data(ZERO);
    let const_zero = EntityTag::const_from_data(ZERO);
    let zero_two = EntityTag::from_data(ZERO);
    let const_zero_two = EntityTag::const_from_data(ZERO);
    let first = EntityTag::from_data(FIRST);
    let const_first = EntityTag::const_from_data(FIRST);
    let first_two = EntityTag::from_data(FIRST);
    let const_first_two = EntityTag::const_from_data(FIRST);
    let second = EntityTag::from_data(SECOND);
    let const_second = EntityTag::const_from_data(SECOND);

    let big = EntityTag::from_data(BIG);
    let const_big = EntityTag::const_from_data(BIG);
    let very_big = EntityTag::from_data(VERY_BIG);
    let const_very_big = EntityTag::const_from_data(VERY_BIG);

    assert_eq!(zero, const_zero);
    assert_eq!(zero, zero_two);
    assert_eq!(zero_two, const_zero_two);
    assert_eq!(first, first_two);
    assert_eq!(first, const_first);
    assert_ne!(first, second);
    assert_eq!(first_two, const_first_two);
    assert_eq!(second, const_second);
    assert_eq!(big, const_big);
    assert_eq!(very_big, const_very_big);
}

#[test]
fn test_etag_size_limit() {
    const MAX: &'static str = "12345678901234567890123456789012345678901234567890123456789012";
    const ABOVE_MAX: &'static str = "123456789012345678901234567890123456789012345678901234567890123";
    assert_eq!(MAX.len(), 62);
    assert_eq!(ABOVE_MAX.len(), 63);

    assert_eq!(format!("\"{}\"", MAX).parse::<EntityTag>().unwrap(), EntityTag::checked_strong(MAX).unwrap());
    assert_eq!(format!("\"{}\"", ABOVE_MAX).parse::<EntityTag>().unwrap_err(), etag::ParseError::Overflow);
}

#[test]
fn test_cmp() {
    const FIRST: &'static str = "FIRST";
    const SECOND: &'static str = "SECOND";

    let etag1 = EntityTag::weak(FIRST);
    let etag2 = EntityTag::weak(FIRST);
    assert!(!etag1.strong_eq(&etag2));
    assert!(etag1.weak_eq(&etag2));
    assert!(etag1.strong_ne(&etag2));
    assert!(!etag1.weak_ne(&etag2));

    let etag1 = EntityTag::weak(FIRST);
    let etag2 = EntityTag::weak(SECOND);
    assert!(!etag1.strong_eq(&etag2));
    assert!(!etag1.weak_eq(&etag2));
    assert!(etag1.strong_ne(&etag2));
    assert!(etag1.weak_ne(&etag2));

    let etag1 = EntityTag::weak(FIRST);
    let etag2 = EntityTag::strong(FIRST);
    assert!(!etag1.strong_eq(&etag2));
    assert!(etag1.weak_eq(&etag2));
    assert!(etag1.strong_ne(&etag2));
    assert!(!etag1.weak_ne(&etag2));

    let etag1 = EntityTag::strong(FIRST);
    let etag2 = EntityTag::strong(FIRST);
    assert!(etag1.strong_eq(&etag2));
    assert!(etag1.weak_eq(&etag2));
    assert!(!etag1.strong_ne(&etag2));
    assert!(!etag1.weak_ne(&etag2));
}

#[test]
fn test_etag_fmt() {
    assert_eq!(format!("{}", EntityTag::strong("foobar")), "\"foobar\"");
    assert_eq!(format!("{}", EntityTag::strong("")), "\"\"");
    assert_eq!(format!("{}", EntityTag::weak("weak-etag")), "W/\"weak-etag\"");
    assert_eq!(format!("{}", EntityTag::weak("\u{0065}")), "W/\"\x65\"");
    assert_eq!(format!("{}", EntityTag::weak("")), "W/\"\"");
}

#[test]
fn test_etag_parse_success() {
    assert_eq!("\"foobar\"".parse::<EntityTag>().unwrap(), EntityTag::strong("foobar"));
    assert_eq!("\"\"".parse::<EntityTag>().unwrap(), EntityTag::strong(""));
    assert_eq!("W/\"weaktag\"".parse::<EntityTag>().unwrap(), EntityTag::weak("weaktag"));
    assert_eq!("W/\"\x65\x62\"".parse::<EntityTag>().unwrap(), EntityTag::weak("\x65\x62"));
    assert_eq!("W/\"\"".parse::<EntityTag>().unwrap(), EntityTag::weak(""));
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
