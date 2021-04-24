//! Simple EntityTag implementation, `no_std` friendly.
//!
//! # Features
//!
//! - `std` - Add `EntityTag::from_file_meta` in order to generate ETag using file's metadata.
//!
//! # Usage
//!
//! ```rust
//! use etag::EntityTag;
//!
//! fn main() {
//!     let my_tag = EntityTag::strong("lolka");
//!     let text_etag = my_tag.to_string();
//!     let parse_tag = text_etag.parse::<EntityTag>().unwrap();
//!
//!     assert!(my_tag.strong_eq(&parse_tag));
//! }
//! ```

#![no_std]
#![deny(warnings)]

#[cfg(feature = "std")]
extern crate std;

use core::fmt::{self, Write};

const SEED: u64 = 0x1000;

type Buffer = str_buf::StrBuf::<64>;

/// An entity tag, defined in [RFC7232](https://tools.ietf.org/html/rfc7232#section-2.3)
///
/// The ETag HTTP response header is an identifier for a specific version of a resource. It allows
/// caches to be more efficient, and saves bandwidth, as a web server does not need to send a full
/// response if the content has not changed. On the other side, if the content has changed, etags
/// are useful to help prevent simultaneous updates of a resource from overwriting each other
/// ("mid-air collisions").
///
/// If the resource at a given URL changes, a new Etag value must be generated. Etags are therefore
/// similar to fingerprints and might also be used for tracking purposes by some servers. A
/// comparison of them allows to quickly determine whether two representations of a resource are the
/// same, but they might also be set to persist indefinitely by a tracking server.
///
/// # Size limit
///
/// In order to avoid allocation, ETag size is limited to 64 characters, which should be sufficient
/// for any hashing mechanism.
///
/// # Format `W/"<etag_value>"`
///
/// - 'W/' (case-sensitive) indicates that a weak validator is used. Weak validators are easy to
/// generate but are far less useful for comparisons. Strong validators are ideal for comparisons
/// but can be very difficult to generate efficiently. Weak Etag values of two representations of
/// the same resources might be semantically equivalent, but not byte-for-byte identical.
///
/// - "<etag_value>" Entity tags uniquely representing the requested resources. They are a string of ASCII
/// characters placed between double quotes (Like "675af34563dc-tr34"). The method by which ETag
/// values are generated is not specified. Oftentimes, a hash of the content, a hash of the last
/// modification timestamp, or just a revision number is used. For example, MDN uses a hash of
/// hexadecimal digits of the wiki content.
///
/// # Comparison
/// To check if two entity tags are equivalent in an application always use the
/// `strong_eq` or `weak_eq` methods based on the context of the Tag. Only use
/// `==` to check if two tags are identical.
///
/// The example below shows the results for a set of entity-tag pairs and
/// both the weak and strong comparison function results:
///
/// | `ETag 1`| `ETag 2`| Strong Comparison | Weak Comparison |
/// |---------|---------|-------------------|-----------------|
/// | `W/"1"` | `W/"1"` | no match          | match           |
/// | `W/"1"` | `W/"2"` | no match          | no match        |
/// | `W/"1"` | `"1"`   | no match          | match           |
/// | `"1"`   | `"1"`   | match             | match           |
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityTag {
    /// Weakness indicator for the tag
    pub weak: bool,
    /// The opaque string in between the DQUOTEs
    tag: Buffer,
}

impl EntityTag {
    /// Constructs a new EntityTag, asserting that it doesn't overflow and valid ASCII string.
    ///
    /// Assertions are performed in debug mode only.
    pub fn new(weak: bool, tag: &str) -> Self {
        let mut result = Self {
            weak,
            tag: Buffer::new(),
        };

        debug_assert!(tag.is_ascii());
        let written = result.tag.push_str(tag);
        debug_assert_eq!(written, tag.len());
        result
    }

    #[inline]
    /// Constructs a new weak EntityTag, using the same checks as `new`.
    pub fn weak(tag: &str) -> Self {
        Self::new(true, tag)
    }

    #[inline]
    /// Constructs a new strong EntityTag, using the same checks as `new`.
    pub fn strong(tag: &str) -> Self {
        Self::new(false, tag)
    }

    /// Constructs a new EntityTag, verifying it's size and whether it includes ASCII.
    pub fn checked_new(weak: bool, tag: &str) -> Result<Self, ParseError> {
        if tag.is_ascii() {
            let mut result = Self {
                weak,
                tag: Buffer::new(),
            };

            match result.tag.push_str(tag) == tag.len() {
                true => Ok(result),
                false => Err(ParseError::Overflow)
            }
        } else {
            Err(ParseError::NotAscii)
        }
    }

    #[inline]
    /// Constructs a new weak EntityTag, using the same checks as `checked_new`.
    pub fn checked_weak(tag: &str) -> Result<Self, ParseError> {
        Self::checked_new(true, tag)
    }

    #[inline]
    /// Constructs a new strong EntityTag, using the same checks as `checked_new`.
    pub fn checked_strong(tag: &str) -> Result<Self, ParseError> {
        Self::checked_new(false, tag)
    }

    #[cfg(feature = "std")]
    /// Creates weak EntityTag from file metadata using modified time and len.
    ///
    /// ## Format:
    ///
    /// `[modified-]<len>`
    pub fn from_file_meta(metadata: &std::fs::Metadata) -> Self {
        let mut tag = Buffer::new();
        let _ = match metadata.modified().map(|modified| modified.duration_since(std::time::UNIX_EPOCH).expect("Modified is earlier than time::UNIX_EPOCH!")) {
            Ok(modified) => write!(tag, "{}.{}-{}", modified.as_secs(), modified.subsec_nanos(), metadata.len()),
            _ => write!(tag, "{}", metadata.len())
        };

        Self {
            weak: true,
            tag
        }
    }

    /// Creates strong EntityTag by hashing provided bytes.
    ///
    /// ## Format:
    ///
    /// `<len>-<hash>`
    pub fn from_hash(bytes: &[u8]) -> Self {
        let hash = wy::def_hash(bytes, SEED);
        let mut tag = Buffer::new();
        let _ = write!(tag, "{}-{}", bytes.len(), hash);

        Self {
            weak: false,
            tag
        }
    }

    /// Get the tag.
    pub fn tag(&self) -> &str {
        self.tag.as_str()
    }

    /// For strong comparison two entity-tags are equivalent if both are not
    /// weak and their opaque-tags match character-by-character.
    pub fn strong_eq(&self, other: &EntityTag) -> bool {
        !self.weak && !other.weak && self.tag.as_str() == other.tag.as_str()
    }

    /// For weak comparison two entity-tags are equivalent if their
    /// opaque-tags match character-by-character, regardless of either or
    /// both being tagged as "weak".
    pub fn weak_eq(&self, other: &EntityTag) -> bool {
        self.tag.as_str() == other.tag.as_str()
    }

    /// The inverse of `EntityTag.strong_eq()`.
    pub fn strong_ne(&self, other: &EntityTag) -> bool {
        !self.strong_eq(other)
    }

    /// The inverse of `EntityTag.weak_eq()`.
    pub fn weak_ne(&self, other: &EntityTag) -> bool {
        !self.weak_eq(other)
    }
}

impl fmt::Display for EntityTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.weak {
            f.write_str("W/")?;
        }

        f.write_char('"')?;
        f.write_str(self.tag.as_str())?;
        f.write_char('"')
    }
}

///Describes possible errors for EntityTag
#[derive(PartialEq, Eq, Debug)]
pub enum ParseError {
    ///Format of EntityTag is invalid
    InvalidFormat,
    ///Tag contains non-ASCII characters
    NotAscii,
    ///Tag doesn't fit buffer.
    Overflow,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => f.write_str("EntityTag uses invalid format"),
            ParseError::NotAscii => f.write_str("EntityTag uses non-ASCII characters"),
            ParseError::Overflow => f.write_str("EntityTag size overflows buffer"),
        }
    }
}

impl core::str::FromStr for EntityTag {
    type Err = ParseError;

    fn from_str(text: &str) -> Result<EntityTag, ParseError> {
        let len = text.len();
        let slice = &text[..];

        if !slice.ends_with('"') || len < 2 {
            return Err(ParseError::InvalidFormat);
        }

        if slice.starts_with('"') {
            let slice = &slice[1..len-1];
            EntityTag::checked_strong(slice)
        } else if len >= 4 && slice.starts_with("W/\"") {
            let slice = &slice[3..len-1];
            EntityTag::checked_weak(slice)
        } else {
            Err(ParseError::InvalidFormat)
        }
    }
}
