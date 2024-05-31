use std::{
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
};

use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use error::ErrInto;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::{error::Error, parts::Parts};

pub type Result<T> = std::result::Result<T, Error>;

pub mod basic_authentication;
pub mod error;
pub mod parts;
pub mod response;

pub trait TryFromJson {
    fn try_from_json<U: AsRef<[u8]>>(slice: U) -> Result<Self>
    where
        Self: Sized;
}

pub trait ToJson {
    fn to_json(&self) -> Result<Vec<u8>>;
}

impl<S: Serialize> ToJson for S {
    fn to_json(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).err_into()
    }
}

pub trait RowExt {
    fn column(&self, column_name: &str) -> Result<&str>;
}

impl RowExt for spin_sdk::sqlite::Row<'_> {
    fn column(&self, column_name: &str) -> Result<&str> {
        self.get::<&str>(column_name)
            .ok_or(Error::Column(column_name.to_owned()))
    }
}

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

impl<T: DeserializeOwned> TryFromJson for T {
    fn try_from_json<U: AsRef<[u8]>>(slice: U) -> Result<T>
    where
        Self: Sized,
    {
        serde_json::from_slice(slice.as_ref()).err_into()
    }
}

fn to_datetime(s: &str) -> Result<DateTime<Utc>> {
    s.parse::<DateTime<Utc>>().err_into()
}

fn to_uuid(s: &str) -> Result<Uuid> {
    s.parse::<Uuid>().err_into()
}

fn to_parts(s: &str) -> Result<Vec<Vec<String>>> {
    s.parse::<Parts>().err_into().map(|p| p.parts())
}

pub struct List<T> {
    pub inner: Vec<T>,
}

impl TryFrom<spin_sdk::sqlite::Row<'_>> for Lyric {
    type Error = Error;

    fn try_from(row: spin_sdk::sqlite::Row<'_>) -> Result<Self> {
        Ok(Self {
            id: row.column("id").map(Into::into)?,
            title: row.column("title").map(Into::into)?,
            parts: row.column("parts").and_then(to_parts)?,
            created: row
                .column("created")
                .and_then(to_datetime)
                .map(Into::into)?,
            modified: row
                .column("modified")
                .and_then(to_datetime)
                .map(Into::into)?,
            etag: row.column("etag").and_then(to_uuid).map(Into::into)?,
        })
    }
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

impl TryFrom<spin_sdk::sqlite::Row<'_>> for Playlist {
    type Error = Error;

    fn try_from(row: spin_sdk::sqlite::Row<'_>) -> Result<Self> {
        Ok(Self {
            id: row.column("id").map(Into::into)?,
            title: row.column("title").map(Into::into)?,
            members: vec![],
            created: row
                .column("created")
                .and_then(to_datetime)
                .map(Into::into)?,
            modified: row
                .column("modified")
                .and_then(to_datetime)
                .map(Into::into)?,
            etag: row.column("etag").and_then(to_uuid).map(Into::into)?,
        })
    }
}

pub struct LyricId(String);

impl LyricId {
    pub fn id(&self) -> String {
        self.0.clone()
    }
}

impl TryFrom<spin_sdk::sqlite::Row<'_>> for LyricId {
    type Error = Error;

    fn try_from(row: spin_sdk::sqlite::Row<'_>) -> Result<Self> {
        row.column("lyric_id").map(String::from).map(LyricId)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaylistPost {
    pub title: String,
    pub members: Vec<String>,
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
    use spin_sdk::sqlite::{QueryResult, RowResult, Value};

    use super::Lyric;

    const UUID: super::Uuid = super::Uuid {
        inner: uuid::uuid!("71795c73-3cdc-49f1-847e-93193232e6c2"),
    };

    const UUID_JSON: &str = "\"F1iFNnPnjRrqdKfCUKPvU1\"";

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

    #[test]
    fn serialization() {
        let s = serde_json::to_string(&UUID).unwrap();
        assert_eq!(s, UUID_JSON);
    }

    #[test]
    fn deserialization() {
        let uuid = serde_json::from_str::<super::Uuid>(UUID_JSON).unwrap();
        assert_eq!(uuid, UUID);
    }

    #[test]
    fn list_from_query_result() {
        let query_result = QueryResult {
            columns: vec![
                "id".to_owned(),
                "title".to_owned(),
                "parts".to_owned(),
                "created".to_owned(),
                "modified".to_owned(),
                "etag".to_owned(),
            ],
            rows: vec![
                RowResult {
                    values: vec![
                        Value::Text("PKc2FHaQoVbJfjsPHwbUX4".to_owned()),
                        Value::Text("Hallo allemaal".to_owned()),
                        Value::Text("Hallo allemaal\nWat fijn dat u bent".to_owned()),
                        Value::Text("2024-05-11T06:38:11.759Z".to_owned()),
                        Value::Text("2024-05-11T06:38:11.759Z".to_owned()),
                        Value::Text("U5jCFGBECj34LSqvZKRz92".to_owned()),
                    ],
                },
                RowResult {
                    values: vec![
                        Value::Text("B3aC2EHKXDGZcjNvvg4Rs5".to_owned()),
                        Value::Text("Sofietje".to_owned()),
                        Value::Text("Zij dronk ranja met een rietje, mijn sofietje\nOp een amsterdams terras".to_owned()),
                        Value::Text("2024-05-11T06:39:11.759Z".to_owned()),
                        Value::Text("2024-05-11T06:39:11.759Z".to_owned()),
                        Value::Text("UBBrNrdTUXT9nMjBrt5dGu".to_owned()),
                    ],
                },
            ],
        };
        let list = query_result
            .rows()
            .map(Lyric::try_from)
            .collect::<Result<Vec<_>, crate::error::Error>>()
            .unwrap();

        for lyric in list {
            println!("{}", lyric.title);
        }
    }
}
