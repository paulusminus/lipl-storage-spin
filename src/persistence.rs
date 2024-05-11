#[cfg(not(target_family = "wasm"))]
use spin_sdk::sqlite::{QueryResult, RowResult};
use spin_sdk::sqlite::{Row, Value};
use std::{fmt::Display, thread::sleep, time::Duration};

use crate::{message, Error, Result};
use model::{error::ErrInto, parts::Parts, Db, Lyric, Playlist, Uuid};

pub enum DbConnection {
    Spin(spin_sdk::sqlite::Connection),
    #[allow(dead_code)]
    #[cfg(not(target_family = "wasm"))]
    Rusqlite(rusqlite::Connection),
}

pub struct Connection(DbConnection);

impl Connection {
    pub(crate) fn try_open_default() -> Result<Self> {
        let connection = spin_sdk::sqlite::Connection::open_default()?;
        message::db_connection_established();
        connection.execute("PRAGMA foreign_keys = ON", &[])?;
        message::foreign_keys();
        Ok(Self(DbConnection::Spin(connection)))
    }

    #[allow(dead_code)]
    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn open_test() -> Result<Self> {
        let connection = rusqlite::Connection::open_in_memory()?;
        connection.execute_batch(include_str!("../migrations.sql"))?;
        connection.execute("PRAGMA foreign_keys = ON", [])?;
        Ok(Self(DbConnection::Rusqlite(connection)))
    }

    fn query<F, S, T>(&self, statement: S, parameters: &[Value], f: F) -> Result<Vec<T>>
    where
        F: Fn(Row) -> Result<T>,
        S: AsRef<str>,
    {
        match &self.0 {
            DbConnection::Spin(connection) => connection
                .execute(statement.as_ref(), parameters)
                .err_into()
                .and_then(|result| result.rows().map(f).collect()),
            #[cfg(not(target_family = "wasm"))]
            DbConnection::Rusqlite(connection) => {
                let mut statement = connection.prepare(statement.as_ref())?;
                for (i, parameter) in parameters
                    .iter()
                    .enumerate()
                    .map(|(i, value)| (i + 1, value))
                {
                    match parameter {
                        Value::Blob(blob) => {
                            statement.raw_bind_parameter(i, blob)?;
                        }
                        Value::Integer(integer) => {
                            statement.raw_bind_parameter(i, integer)?;
                        }
                        Value::Null => {
                            statement.raw_bind_parameter(i, Option::<String>::None)?;
                        }
                        Value::Real(float) => {
                            statement.raw_bind_parameter(i, float)?;
                        }
                        Value::Text(s) => {
                            statement.raw_bind_parameter(i, s)?;
                        }
                    };
                }
                let columns = statement
                    .column_names()
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<_>>();
                let mut query_result = QueryResult {
                    columns: columns.clone(),
                    rows: vec![],
                };
                let mut rows = statement.raw_query();
                while let Some(row) = rows.next()? {
                    let mut row_result = RowResult { values: vec![] };
                    for column in columns.iter() {
                        let field = row.get::<&str, String>(column)?;
                        row_result.values.push(Value::Text(field));
                    }
                    query_result.rows.push(row_result);
                }
                query_result.rows().map(f).collect()
            }
        }
    }

    fn execute<S>(&self, statement: S, parameters: &[Value]) -> Result<i64>
    where
        S: AsRef<str>,
    {
        match &self.0 {
            DbConnection::Spin(connection) => {
                connection.execute(statement.as_ref(), parameters)?;
                let changes = connection.execute(sql::SQL_GET_CHANGES, &[])?;
                match changes.rows.first().cloned() {
                    Some(row) => {
                        // using i64 is crucial !!!
                        let count = row
                            .get::<i64>(0)
                            .ok_or(Error::MissingColumn("changes ()"))?;
                        Ok(count)
                    }
                    None => Ok(0),
                }
            }
            #[cfg(not(target_family = "wasm"))]
            DbConnection::Rusqlite(connection) => {
                let mut statement = connection.prepare(statement.as_ref())?;
                for (i, parameter) in parameters
                    .iter()
                    .enumerate()
                    .map(|(i, value)| (i + 1, value))
                {
                    match parameter {
                        Value::Blob(blob) => {
                            statement.raw_bind_parameter(i, blob)?;
                        }
                        Value::Integer(integer) => {
                            statement.raw_bind_parameter(i, integer)?;
                        }
                        Value::Null => {
                            statement.raw_bind_parameter(i, Option::<String>::None)?;
                        }
                        Value::Real(float) => {
                            statement.raw_bind_parameter(i, float)?;
                        }
                        Value::Text(s) => {
                            statement.raw_bind_parameter(i, s)?;
                        }
                    };
                }
                let result = statement.raw_execute()?;
                Ok(result.try_into().unwrap())
            }
        }
    }

    pub(crate) fn begin_transaction(&self) -> Result<()> {
        self.execute(sql::SQL_BEGIN_TRANSACTION, &[]).map(|_| ())
    }

    pub(crate) fn roll_back(&self) -> Result<()> {
        self.execute(sql::SQL_ROLLBACK, &[]).map(|_| ())
    }

    pub(crate) fn commit(&self) -> Result<()> {
        self.execute(sql::SQL_COMMIT, &[]).map(|_| ())
    }

    pub(crate) fn get_lyric_list(&self) -> Result<Vec<Lyric>> {
        self.query(sql::SQL_GET_LYRIC_LIST, &[], |r| Lyric::try_from(r))
    }

    pub(crate) fn get_lyric<D>(&self, id: D) -> Result<Option<Lyric>>
    where
        D: Display,
    {
        let params = vec![Value::Text(id.to_string())];
        self.query(sql::SQL_GET_LYRIC, &params, |r| Lyric::try_from(r))
            .map(|result| result.first().cloned())
    }

    pub(crate) fn delete_lyric<D>(&self, id: D) -> Result<bool>
    where
        D: Display,
    {
        let params = vec![Value::Text(id.to_string())];
        self.execute(sql::SQL_DELETE_LYRIC, &params).map(|c| c > 0)
    }

    pub(crate) fn update_lyric(&self, lyric: &Lyric) -> Result<bool> {
        let params = vec![
            Value::Text(lyric.title.clone()),
            Value::Text(Parts::from(lyric.parts.clone()).to_text()),
            Value::Text(lyric.id.clone()),
        ];
        self.execute(sql::SQL_UPDATE_LYRIC, &params).map(|c| c > 0)
    }

    pub(crate) fn insert_lyric(&self, lyric: &Lyric) -> Result<bool> {
        let params = vec![
            Value::Text(lyric.id.clone()),
            Value::Text(lyric.title.clone()),
            Value::Text(Parts::from(lyric.parts.clone()).to_text()),
            Value::Text(Uuid::default().to_string()),
        ];
        self.execute(sql::SQL_INSERT_LYRIC, &params).map(|c| c > 0)
    }

    fn get_playlist_members<D>(&self, playlist_id: D) -> Result<Vec<String>>
    where
        D: Display,
    {
        self.query(
            sql::SQL_GET_MEMBER_LYRICS,
            &[Value::Text(playlist_id.to_string())],
            |r| {
                r.get::<&str>("lyric_id")
                    .ok_or(Error::MissingLyricId)
                    .map(String::from)
            },
        )
    }

    pub(crate) fn get_playlist_list(&self) -> Result<Vec<Playlist>> {
        let mut playlists =
            self.query(sql::SQL_GET_PLAYLIST_LIST, &[], |r| Playlist::try_from(r))?;

        for playlist in playlists.iter_mut() {
            playlist.members = self.get_playlist_members(playlist.id.clone())?;
        }
        Ok(playlists)
    }

    pub(crate) fn get_playlist<D>(&self, id: D) -> Result<Option<Playlist>>
    where
        D: Display,
    {
        let params = vec![Value::Text(id.to_string())];
        let result = self.query(sql::SQL_GET_PLAYLIST, &params, |r| Playlist::try_from(r))?;
        match result.first().cloned() {
            Some(mut playlist) => {
                playlist.members = self.get_playlist_members(&playlist.id)?;
                Ok(Some(playlist.clone()))
            }
            None => Ok(None),
        }
    }

    pub(crate) fn delete_playlist<D>(&self, id: D) -> Result<bool>
    where
        D: Display,
    {
        let params = vec![Value::Text(id.to_string())];
        self.execute(sql::SQL_DELETE_PLAYLIST, &params)
            .map(|count| count > 0)
    }

    pub(crate) fn delete_members<F>(&self, playlist_id: &String, onerror: F) -> Result<i64>
    where
        F: Fn() -> Result<()>,
    {
        self.execute(sql::SQL_DELETE_MEMBER, &[Value::Text(playlist_id.clone())])
            .inspect_err(|_| {
                if onerror().is_err() {
                    message::delete_member_failure(playlist_id);
                }
            })
    }

    pub(crate) fn insert_members<F>(
        &self,
        playlist_id: &String,
        lyric_ids: &[String],
        onerror: F,
    ) -> Result<()>
    where
        F: Fn() -> Result<()>,
    {
        let mut i: i64 = 0;
        for lyric_id in lyric_ids {
            i += 1;
            self.execute(
                sql::SQL_INSERT_MEMBER,
                &[
                    Value::Text(playlist_id.clone()),
                    Value::Text(lyric_id.clone()),
                    Value::Integer(i),
                ],
            )
            .inspect_err(|_| {
                if onerror().is_err() {
                    message::insert_member_failure(lyric_id, playlist_id);
                }
            })?;
        }
        Ok(())
    }

    pub(crate) fn update_playlist(&self, playlist: &Playlist) -> Result<()> {
        self.begin_transaction()?;

        self.delete_members(&playlist.id, || self.roll_back())?;

        self.execute(
            sql::SQL_UPDATE_PLAYLIST,
            &[
                Value::Text(playlist.title.clone()),
                Value::Text(Uuid::default().to_string()),
                Value::Text(playlist.id.clone()),
            ],
        )
        .inspect_err(|_| {
            if self.roll_back().is_err() {
                message::update_playlist_failure(&playlist.id);
            }
        })?;

        self.insert_members(&playlist.id, &playlist.members, || self.roll_back())?;
        self.commit()?;

        Ok(())
    }

    pub(crate) fn insert_playlist(&self, playlist: &Playlist) -> Result<()> {
        self.begin_transaction()?;

        self.execute(
            sql::SQL_INSERT_PLAYLIST,
            &[
                Value::Text(playlist.id.clone()),
                Value::Text(playlist.title.clone()),
                Value::Text(Uuid::default().to_string()),
            ],
        )
        .inspect_err(|error| {
            if self.roll_back().is_err() {
                message::insert_playlist_failure(error);
            };
        })?;

        self.insert_members(&playlist.id, &playlist.members, || self.roll_back())?;
        self.commit()?;

        Ok(())
    }

    pub(crate) fn replace_db(&self, db: &Db) -> Result<()> {
        self.execute("DELETE FROM playlist", &[])?;
        self.execute("DELETE FROM lyric", &[])?;
        self.execute("DELETE FROM member", &[])?;

        for lyric in db.lyrics.iter() {
            self.insert_lyric(lyric)?;
            sleep(Duration::from_millis(2));
        }

        for playlist in db.playlists.iter() {
            self.insert_playlist(playlist)?;
            sleep(Duration::from_millis(2));
        }

        Ok(())
    }
}

mod sql {
    pub const SQL_GET_CHANGES: &str = "SELECT changes()";
    pub const SQL_BEGIN_TRANSACTION: &str = "BEGIN TRANSACTION";
    pub const SQL_ROLLBACK: &str = "ROLLBACK";
    pub const SQL_COMMIT: &str = "COMMIT";

    pub const SQL_GET_LYRIC_LIST: &str =
        "SELECT id, title, parts, created, modified, etag FROM lyric ORDER BY title";
    pub const SQL_GET_LYRIC: &str =
        "SELECT id, title, parts, created, modified, etag FROM lyric WHERE Id=?";
    pub const SQL_INSERT_LYRIC: &str = "INSERT INTO lyric (id, title, parts, created, modified, etag) VALUES (?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), ?)";
    pub const SQL_UPDATE_LYRIC: &str = "UPDATE lyric SET title=?, parts=?, modified=strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE Id=?";
    pub const SQL_DELETE_LYRIC: &str = "DELETE FROM lyric WHERE Id=?";

    pub const SQL_GET_PLAYLIST_LIST: &str =
        "SELECT id, title, created, modified, etag FROM playlist ORDER BY title";
    pub const SQL_GET_PLAYLIST: &str =
        "SELECT id, title, created, modified, etag FROM playlist WHERE Id=?";

    pub const SQL_INSERT_PLAYLIST: &str = "INSERT INTO playlist (id, title, created, modified, etag) VALUES (?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), ?)";
    pub const SQL_UPDATE_PLAYLIST: &str = "UPDATE playlist SET title = ?, modified = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), etag = ? WHERE id = ?";
    pub const SQL_DELETE_PLAYLIST: &str = "DELETE FROM playlist WHERE Id=?";

    pub const SQL_GET_MEMBER_LYRICS: &str =
        "SELECT lyric_id FROM member WHERE playlist_id = ? ORDER BY ordering";
    pub const SQL_INSERT_MEMBER: &str =
        "INSERT INTO member (playlist_id, lyric_id, ordering) VALUES (?, ?, ?)";
    pub const SQL_DELETE_MEMBER: &str = "DELETE FROM member WHERE playlist_id = ?";
}

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use model::{error::Error, Db, Lyric, Playlist, TryFromJson, Uuid};
    use spin_sdk::sqlite::Row;

    use super::Connection;

    #[test]
    fn open_database() {
        Connection::open_test().unwrap();
    }

    #[test]
    fn insert_lyric() {
        let connection = Connection::open_test().unwrap();

        let id = Uuid::default().to_string();
        let mut lyric = Lyric::new(id.clone(), "Zie maar hoe je het doet".to_owned(), vec![]);
        connection.insert_lyric(&lyric).unwrap();

        let stored_lyric = connection.get_lyric(id.clone()).unwrap().unwrap();
        assert!(stored_lyric.created.is_some());
        assert!(stored_lyric.modified.is_some());
        assert!(stored_lyric.etag.is_some());

        lyric.title = "Hallo allemaal".to_owned();
        thread::sleep(Duration::from_millis(5));
        if connection.update_lyric(&lyric).unwrap() {
            let lyric = connection.get_lyric(id.clone()).unwrap().unwrap();
            println!("created: {}", lyric.created.unwrap());
            println!("modified: {}", lyric.modified.unwrap());
        };
    }

    #[test]
    fn insert_playlist() {
        let connection = Connection::open_test().unwrap();

        let lyric_id = Uuid::default().to_string();
        let lyric = Lyric::new(
            lyric_id.clone(),
            "Zie maar hoe je het doet".to_owned(),
            vec![],
        );

        assert!(lyric.created.is_none());
        assert!(lyric.modified.is_none());
        assert!(lyric.etag.is_none());

        connection.insert_lyric(&lyric).unwrap();

        let playlist_id = Uuid::default().to_string();
        let playlist = Playlist::new(
            playlist_id.clone(),
            "Alles".to_owned(),
            vec![lyric_id.clone()],
        );
        connection.insert_playlist(&playlist).unwrap();

        let stored_playlist = connection
            .get_playlist(playlist_id.clone())
            .unwrap()
            .unwrap();
        assert!(stored_playlist.created.is_some());
        assert!(stored_playlist.modified.is_some());
        assert!(stored_playlist.etag.is_some());

        connection.delete_lyric(lyric_id.clone()).unwrap();

        let without_lyric = connection
            .get_playlist(playlist_id.clone())
            .unwrap()
            .unwrap();
        assert!(without_lyric.members.is_empty());
    }

    #[test]
    fn show_tables() {
        let connection = Connection::open_test().unwrap();

        struct Table {
            name: String,
        }

        impl TryFrom<Row<'_>> for Table {
            type Error = Error;

            fn try_from(row: Row<'_>) -> Result<Self, Self::Error> {
                row.get::<&str>("name")
                    .ok_or(Error::MissingColumn("name"))
                    .map(String::from)
                    .map(|s| Self { name: s })
            }
        }
        let tables = connection
            .query(
                "SELECT name FROM sqlite_schema WHERE type ='table' AND  name NOT LIKE 'sqlite_%';",
                &[],
                |r| Table::try_from(r),
            )
            .unwrap();

        let mut names = tables
            .into_iter()
            .map(|table| table.name)
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            names,
            vec![
                "list_etag".to_owned(),
                "lyric".to_owned(),
                "member".to_owned(),
                "playlist".to_owned()
            ]
        );
    }

    #[test]
    fn import_db() {
        let connection = Connection::open_test().unwrap();

        let db = Db::try_from_json(include_bytes!("../data/db.json")).unwrap();
        connection.replace_db(&db).unwrap();

        let lyrics = connection.get_lyric_list().unwrap();
        assert_eq!(lyrics.len(), 57);

        let playlists = connection.get_playlist_list().unwrap();
        assert_eq!(playlists.len(), 1);
    }
}
