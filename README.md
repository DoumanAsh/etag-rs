etag-rs
====================

[![Crates.io](https://img.shields.io/crates/v/etag.svg)](https://crates.io/crates/etag)
[![Docs.rs](https://docs.rs/etag)](https://docs.rs/etag)

ETag Trait implementation for various types.

For file's Metadata it uses following format `<modified>-<size>`

For other types it uses `<len>-<hash>` that relies on `std::hash::Hash` and `DefaultHasher` of HashMap.

## Usage
```rust
extern crate etag;

use etag::Etag;

fn main() {
    println!("ETag for string={}", "string".etag());
}
```
