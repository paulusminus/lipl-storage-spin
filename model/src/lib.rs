use std::{
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
};

use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use error::ErrInto;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(feature = "response")]
pub mod basic_authentication;
#[cfg(feature = "response")]
pub mod convert;
pub mod error;
pub mod parts;
#[cfg(feature = "response")]
pub mod response;


pub trait Etag {
    fn etag(&self) -> String;
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub struct Lyric {
    pub id: String,
    pub title: String,
    pub parts: Vec<Vec<String>>,
    #[serde(skip)]
    pub created: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub modified: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub etag: Option<Uuid>,
}

impl Lyric {
    pub fn new(id: String, title: String, parts: Vec<Vec<String>>) -> Self {
        Self {
            id,
            title,
            parts,
            created: None,
            modified: None,
            etag: None,
        }
    }
}

pub struct List<T> {
    pub inner: Vec<T>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LyricPost {
    pub title: String,
    pub parts: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub members: Vec<String>,
    #[serde(skip)]
    pub created: Option<chrono::DateTime<Utc>>,
    #[serde(skip)]
    pub modified: Option<chrono::DateTime<Utc>>,
    #[serde(skip)]
    pub etag: Option<Uuid>,
}

impl Playlist {
    pub fn new(id: String, title: String, members: Vec<String>) -> Self {
        Self {
            id,
            title,
            members,
            created: None,
            modified: None,
            etag: None,
        }
    }
}

impl<T: Hash> Etag for T {
    fn etag(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        let bytes = hash.to_le_bytes();
        STANDARD_NO_PAD.encode(bytes)
    }
}


pub struct LyricId(pub String);


#[derive(Debug, Deserialize, Serialize)]
pub struct PlaylistPost {
    pub title: String,
    pub members: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    #[serde(skip)]
    pub password: String,
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.id, self.name)
    }
}

#[derive(Debug, Deserialize, Hash, Serialize)]
pub struct Db {
    pub lyrics: Vec<Lyric>,
    pub playlists: Vec<Playlist>,
}

#[derive(Clone, Debug, DeserializeFromStr, Hash, SerializeDisplay, PartialEq)]
pub struct Uuid {
    inner: uuid::Uuid,
}

impl Uuid {
    pub fn from_uuid_str(s: &str) -> Result<Self> {
        s.parse::<uuid::Uuid>()
            .err_into()
            .map(|uuid| Self { inner: uuid })
    }
}

impl FromStr for Uuid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        bs58::decode(s)
            .into_vec()
            .err_into()
            .and_then(|v| uuid::Uuid::from_slice(v.as_slice()).err_into())
            .map(|uuid| Self { inner: uuid })
    }
}

impl Default for Uuid {
    fn default() -> Self {
        Self {
            inner: uuid::Uuid::new_v4(),
        }
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", bs58::encode(self.inner.as_bytes()).into_string())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn from_option() {
        let x = 8;
        let y: Option<i32> = x.into();
        assert_eq!(y, Some(8));
    }

    #[test]
    fn new() {
        let uuid = super::Uuid::default();
        let s = uuid.to_string();
        println!("{s}");
        let new_uuid = s.parse::<super::Uuid>().unwrap();
        assert_eq!(uuid, new_uuid);
    }
}
