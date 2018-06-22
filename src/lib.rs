//! Simple EntityTag implementation.
//!
use std::fs;
use std::fmt;
use std::time;
use std::error;
use std::hash::{
    Hash,
    Hasher
};
use std::collections::hash_map::DefaultHasher;

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
    tag: String,
}

impl EntityTag {
    /// Constructs a new EntityTag.
    ///
    /// As tag characters must be in ASCII assert
    /// is included to check for it.
    pub fn new(weak: bool, tag: String) -> Self {
        assert!(tag.is_ascii());
        EntityTag { weak, tag }
    }

    /// Constructs a new weak EntityTag.
    pub fn weak(tag: String) -> Self {
        Self::new(true, tag)
    }

    /// Constructs a new strong EntityTag.
    pub fn strong(tag: String) -> Self {
        Self::new(false, tag)
    }

    /// Creates weak EntityTag from file metadata using modified time and len.
    pub fn from_file_meta(metadata: &fs::Metadata) -> Self {
        let tag = match metadata.modified().map(|modified| modified.duration_since(time::UNIX_EPOCH).expect("Modified is earlier than time::UNIX_EPOCH!")) {
            Ok(modified) => format!("{}.{}-{}", modified.as_secs(), modified.subsec_nanos(), metadata.len()),
            _ => format!("{}", metadata.len())
        };

        Self {
            weak: true,
            tag
        }
    }

    /// Creates strong EntityTag by hashing provided bytes.
    pub fn from_hash(bytes: &[u8]) -> Self {
        let mut hasher = DefaultHasher::default();
        bytes.hash(&mut hasher);
        let tag = format!("{}-{}", bytes.len(), hasher.finish());

        Self {
            weak: false,
            tag
        }
    }

    /// Get the tag.
    pub fn tag(&self) -> &str {
        self.tag.as_ref()
    }

    /// Set the tag.
    pub fn set_tag(&mut self, tag: String) {
        self.tag = tag
    }

    /// For strong comparison two entity-tags are equivalent if both are not
    /// weak and their opaque-tags match character-by-character.
    pub fn strong_eq(&self, other: &EntityTag) -> bool {
        !self.weak && !other.weak && self.tag == other.tag
    }

    /// For weak comparison two entity-tags are equivalent if their
    /// opaque-tags match character-by-character, regardless of either or
    /// both being tagged as "weak".
    pub fn weak_eq(&self, other: &EntityTag) -> bool {
        self.tag == other.tag
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
        match self.weak {
            true => write!(f, "W/\"{}\"", self.tag),
            false => write!(f, "\"{}\"", self.tag),
        }
    }
}

///Describes possible errors for EntityTag
#[derive(Debug)]
pub enum ParseError {
    ///Format of EntityTag is invalid
    InvalidFormat,
    ///Tag contains non-ASCII characters
    NotAscii
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "EntityTag uses invalid format"),
            ParseError::NotAscii => write!(f, "EntityTag uses non-ASCII characters")
        }
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match self {
            ParseError::InvalidFormat => "EntityTag uses invalid format",
            ParseError::NotAscii => "EntityTag uses non-ASCII characters"
        }
    }
}

impl std::str::FromStr for EntityTag {
    type Err = ParseError;

    fn from_str(text: &str) -> Result<EntityTag, ParseError> {
        let len = text.len();
        let slice = &text[..];

        if !slice.ends_with('"') || len < 2 {
            return Err(ParseError::InvalidFormat);
        }

        if slice.starts_with('"') {
            let slice = &slice[1..len-1];
            match slice.is_ascii() {
                true => Ok(EntityTag::strong(slice.to_string())),
                false => Err(ParseError::NotAscii)
            }
        } else if len >= 4 && slice.starts_with("W/\"") {
            let slice = &slice[3..len-1];
            match slice.is_ascii() {
                true => Ok(EntityTag::weak(slice.to_string())),
                false => Err(ParseError::NotAscii)
            }
        } else {
            Err(ParseError::InvalidFormat)
        }
    }
}
