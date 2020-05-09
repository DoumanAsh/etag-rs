etag-rs
====================

![Build](https://github.com/DoumanAsh/etag-rs/workflows/Rust/badge.svg?branch=master)
[![Crates.io](https://img.shields.io/crates/v/etag.svg)](https://crates.io/crates/etag)
[![Docs.rs](https://docs.rs/etag/badge.svg)](https://docs.rs/etag)

Simple EntityTag implementation.

# Usage

```rust
use etag::EntityTag;

fn main() {
    let my_tag = EntityTag::strong("lolka".to_owned());
    let text_etag = my_tag.to_string();
    let parse_tag = text_etag.parse::<EntityTag>().unwrap();

    assert!(my_tag.strong_eq(&parse_tag));
}
```
