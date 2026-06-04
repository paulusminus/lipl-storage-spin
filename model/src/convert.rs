use crate::{Error, Lyric, LyricId, Playlist, Result, User, Uuid, error::ErrInto, parts::Parts};
use chrono::{DateTime, Utc};

pub trait RowExt {
    fn column(&self, column_index: usize, column_name: &str) -> Result<String>;
}

impl RowExt for spin_sdk::sqlite::RowResult {
    fn column(&self, column_index: usize, column_name: &str) -> Result<String> {
        self.get::<&str>(column_index)
            .map(String::from)
            .ok_or(Error::Column(format!(
                "Problem converting value in column {column_name}"
            )))
    }
}

fn to_datetime(s: String) -> Result<DateTime<Utc>> {
    s.parse::<DateTime<Utc>>().err_into()
}

fn to_uuid(s: String) -> Result<Uuid> {
    s.parse::<Uuid>().err_into()
}

fn to_parts(s: String) -> Result<Vec<Vec<String>>> {
    s.parse::<Parts>().err_into().map(|p| p.parts())
}

impl TryFrom<spin_sdk::sqlite::RowResult> for Lyric {
    type Error = Error;

    fn try_from(row: spin_sdk::sqlite::RowResult) -> Result<Self> {
        Ok(Self {
            id: row.column(0, "id")?,
            title: row.column(1, "title")?,
            parts: row.column(2, "parts").and_then(to_parts)?,
            created: row
                .column(3, "created")
                .and_then(to_datetime)
                .map(Into::into)?,
            modified: row
                .column(4, "modified")
                .and_then(to_datetime)
                .map(Into::into)?,
            etag: row.column(5, "etag").and_then(to_uuid).map(Into::into)?,
        })
    }
}

impl TryFrom<spin_sdk::sqlite::RowResult> for Playlist {
    type Error = Error;

    fn try_from(row: spin_sdk::sqlite::RowResult) -> Result<Self> {
        Ok(Self {
            id: row.column(0, "id")?,
            title: row.column(1, "title")?,
            members: vec![],
            created: row
                .column(2, "created")
                .and_then(to_datetime)
                .map(Into::into)?,
            modified: row
                .column(3, "created")
                .and_then(to_datetime)
                .map(Into::into)?,
            etag: row.column(4, "etag").and_then(to_uuid).map(Into::into)?,
        })
    }
}

impl TryFrom<spin_sdk::sqlite::RowResult> for LyricId {
    type Error = Error;

    fn try_from(row: spin_sdk::sqlite::RowResult) -> Result<Self> {
        row.column(0, "id").map(LyricId)
    }
}

impl TryFrom<spin_sdk::sqlite::RowResult> for User {
    type Error = Error;

    fn try_from(row: spin_sdk::sqlite::RowResult) -> Result<Self> {
        Ok(Self {
            id: row.column(0, "id")?,
            name: row.column(1, "name")?,
            password: row.column(2, "password")?,
        })
    }
}

#[cfg(test)]
mod tests {
    // use spin_sdk::sqlite::{QueryResult, RowResult, Value};

    // use crate::Lyric;

    #[test]
    fn list_from_query_result() {
        // let query_result = QueryResult {
        //     columns: vec![
        //         "id".to_owned(),
        //         "title".to_owned(),
        //         "parts".to_owned(),
        //         "created".to_owned(),
        //         "modified".to_owned(),
        //         "etag".to_owned(),
        //     ],
        //     rows: vec![
        //         RowResult {
        //             values: vec![
        //                 Value::Text("PKc2FHaQoVbJfjsPHwbUX4".to_owned()),
        //                 Value::Text("Hallo allemaal".to_owned()),
        //                 Value::Text("Hallo allemaal\nWat fijn dat u bent".to_owned()),
        //                 Value::Text("2024-05-11T06:38:11.759Z".to_owned()),
        //                 Value::Text("2024-05-11T06:38:11.759Z".to_owned()),
        //                 Value::Text("U5jCFGBECj34LSqvZKRz92".to_owned()),
        //             ],
        //         },
        //         RowResult {
        //             values: vec![
        //                 Value::Text("B3aC2EHKXDGZcjNvvg4Rs5".to_owned()),
        //                 Value::Text("Sofietje".to_owned()),
        //                 Value::Text("Zij dronk ranja met een rietje, mijn sofietje\nOp een amsterdams terras".to_owned()),
        //                 Value::Text("2024-05-11T06:39:11.759Z".to_owned()),
        //                 Value::Text("2024-05-11T06:39:11.759Z".to_owned()),
        //                 Value::Text("UBBrNrdTUXT9nMjBrt5dGu".to_owned()),
        //             ],
        //         },
        //     ],
        // };
        // let list = query_result
        //     .rows()
        //     .map(Lyric::try_from)
        //     .collect::<Result<Vec<_>, crate::error::Error>>()
        //     .unwrap();

        // for lyric in list {
        //     println!("{}", lyric.title);
        // }
    }
}
