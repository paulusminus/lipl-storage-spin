use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};

use crate::{error::ErrInto, parts::Parts, Error, Lyric, LyricId, Playlist, Result, User, Uuid};

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

impl<T: DeserializeOwned> TryFromJson for T {
    fn try_from_json<U: AsRef<[u8]>>(slice: U) -> Result<T>
    where
        Self: Sized,
    {
        serde_json::from_slice(slice.as_ref()).err_into()
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

fn to_datetime(s: &str) -> Result<DateTime<Utc>> {
    s.parse::<DateTime<Utc>>().err_into()
}

fn to_uuid(s: &str) -> Result<Uuid> {
    s.parse::<Uuid>().err_into()
}

fn to_parts(s: &str) -> Result<Vec<Vec<String>>> {
    s.parse::<Parts>().err_into().map(|p| p.parts())
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

impl TryFrom<spin_sdk::sqlite::Row<'_>> for LyricId {
    type Error = Error;

    fn try_from(row: spin_sdk::sqlite::Row<'_>) -> Result<Self> {
        row.column("lyric_id").map(String::from).map(LyricId)
    }
}

impl TryFrom<spin_sdk::sqlite::Row<'_>> for User {
    type Error = Error;

    fn try_from(row: spin_sdk::sqlite::Row<'_>) -> Result<Self> {
        Ok(Self {
            id: row.column("id").map(Into::into)?,
            name: row.column("name").map(Into::into)?,
            password: row.column("password").map(Into::into)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use spin_sdk::sqlite::{QueryResult, RowResult, Value};

    use crate::Lyric;

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