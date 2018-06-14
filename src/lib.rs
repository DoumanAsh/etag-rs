//! ETag Trait implementation for various types.
//!
//! For file's Metadata it uses following format `<modified>-<size>`
//!
//! For other types it uses `<len>-<hash>` that relies on `std::hash::Hash` and `DefaultHasher` of HashMap.
//!
//! ## Usage
//! ```rust
//! extern crate etag;
//!
//! use etag::Etag;
//!
//! fn main() {
//!     println!("ETag for string={}", "string".etag());
//! }
//! ```

use std::fs;
use std::time;
use std::hash::{
    Hash,
    Hasher
};
use std::collections::hash_map::DefaultHasher;

///Trait that provides calculation of ETag field for type.
pub trait Etag {
    ///Calculates ETag value.
    fn etag(&self) -> String;
}

impl Etag for fs::Metadata {
    ///Calculates ETag from Metadata in following format `<modified>-<size>`.
    ///
    ///Note that if modified is not available then it is omitted.
    fn etag(&self) -> String {
        match self.modified().map(|modified| modified.duration_since(time::UNIX_EPOCH).expect("Modified is earlier than time::UNIX_EPOCH!")) {
            Ok(modified) => format!("{}.{}-{}", modified.as_secs(), modified.subsec_nanos(), self.len()),
            _ => format!("{}", self.len())
        }
    }
}

macro_rules! impl_with_hasher
{
    ($($t:ty), +) => {
        $(
            impl Etag for $t {
                fn etag(&self) -> String {
                    let mut hasher = DefaultHasher::default();
                    self.hash(&mut hasher);
                    format!("{}-{}", self.len(), hasher.finish())
                }
            }
        )+
    };
}

impl_with_hasher!(str, String, [u8]);
