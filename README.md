etag-rs
====================

[![Build Status](https://travis-ci.org/DoumanAsh/etag-rs.svg?branch=master)](https://travis-ci.org/DoumanAsh/etag-rs)
[![Crates.io](https://img.shields.io/crates/v/etag.svg)](https://crates.io/crates/etag)
[![Docs.rs](https://docs.rs/etag/badge.svg)](https://docs.rs/etag)

Simple EntityTag implementation.

# Usage

```rust
extern crate etag;

use etag::EntityTag;

fn main() {
    let my_tag = EntityTag::strong("lolka".to_owned());
    let text_etag = my_tag.to_string();
    let parse_tag = text_etag.parse::<EntityTag>().unwrap();

    assert!(my_tag.strong_eq(&parse_tag));
}
```
