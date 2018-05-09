extern crate url;

use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer, Error as DeError, Unexpected};

use self::url::{Url, ParseError::{self as UrlParseError, RelativeUrlWithoutBase}};

use std::path::Path;
use std::str::FromStr;

/// A generalized path object. Can be local or remote, absolute or relative. Data urls are not yet supported.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Name {
    /// An absolute path.
    Absolute(Url),
    /// A relative path.
    Relative(String),
    // TODO: data urls
}

impl Name {
    /// Create a path from the given URI string.
    pub fn new(val: &str) -> Result<Name, UrlParseError> {
        Ok(match Url::parse(val) {
            Ok(url) => Absolute(url),
            Err(RelativeUrlWithoutBase) => Relative(val.to_owned()),
            Err(e) => return Err(e)
        })
    }

    /// Create a new file path.
    pub fn file<P: AsRef<Path>>(path: P) -> Name {
        match Url::from_file_path(&path) {
            Ok(url) => Name::Absolute(url),
            Err(()) => Name::Relative(path.as_ref().to_string_lossy().to_string()),
        }
    }

    /// If the path is relative, join it to the given base path. Otherwise return it.
    pub fn within(self, base: &Url) -> Result<Url, UrlParseError> {
        match self {
            Name::Absolute(url) => Ok(url),
            Name::Relative(path) => base.join(path.as_str()),
        }
    }
}

impl<'a> From<&'a Path> for Name {
    fn from(path: &Path) -> Name {
        Name::file(path)
    }
}

impl From<Url> for Name {
    fn from(url: Url) -> Name {
        Name::Absolute(url)
    }
}

impl FromStr for Name {
    type Err = UrlParseError;

    fn from_str(s: &str) -> Result<Name, UrlParseError> {
        Name::new(s)
    }
}

use self::Name::*;

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Absolute(ref url) => serializer.serialize_str(url.as_ref()),
            Relative(ref path) => serializer.serialize_str(path),
        }
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Name, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = <&str as Deserialize>::deserialize(deserializer)?;
        match Name::from_str(val) {
            Ok(name) => Ok(name),
            Err(e) => return Err(D::Error::invalid_value(
                Unexpected::Str(val),
                &format!("url: {}", e).as_ref()
            ))
        }
    }
}
