use spin_sdk::sqlite::Value;
use std::fmt::Display;

use crate::{message, Error, Result};
use model::{error::ErrInto, parts::Parts, Db, Lyric, LyricId, Playlist, Uuid};

trait MapFirst<T: Clone> {
    fn map_first(self) -> Result<Option<T>>;
}

impl<T: Clone> MapFirst<T> for Result<Vec<T>> {
    fn map_first(self) -> Result<Option<T>> {
        self.map(|list| list.first().cloned())
    }
}

trait MapToUnit {
    fn map_to_unit(self) -> Result<()>;
}

impl<T> MapToUnit for Result<T> {
    fn map_to_unit(self) -> Result<()> {
        self.map(|_| ())
    }
}

pub struct Connection(spin_sqlite_connection::DbConnection<Error>);

impl Connection {
    pub fn try_open_default(migrations: Option<&'static str>) -> Result<Self> {
        let connection =
            spin_sqlite_connection::DbConnection::try_open_default(migrations).map(Self)?;
        message::db_connection_established();
        connection.0.execute("PRAGMA foreign_keys = ON", &[])?;
        Ok(connection)
    }

    pub fn begin_transaction(&self) -> Result<()> {
        self.0
            .execute(sql::SQL_BEGIN_TRANSACTION, &[])
            .map_to_unit()
    }

    pub fn roll_back(&self) -> Result<()> {
        self.0.execute(sql::SQL_ROLLBACK, &[]).map_to_unit()
    }

    pub fn commit(&self) -> Result<()> {
        self.0.execute(sql::SQL_COMMIT, &[]).map_to_unit()
    }

    pub fn get_lyric_list(&self) -> Result<Vec<Lyric>> {
        self.0.query::<Lyric>(sql::SQL_GET_LYRIC_LIST, &[])
    }

    pub fn get_lyric<D>(&self, id: D) -> Result<Option<Lyric>>
    where
        D: Display,
    {
        let params = vec![Value::Text(id.to_string())];
        self.0
            .query::<Lyric>(sql::SQL_GET_LYRIC, &params)
            .map_first()
            .err_into()
    }

    pub fn delete_lyric<D>(&self, id: D) -> Result<bool>
    where
        D: Display,
    {
        let params = vec![Value::Text(id.to_string())];
        self.0
            .execute(sql::SQL_DELETE_LYRIC, &params)
            .map(|c| c > 0)
    }

    pub fn update_lyric(&self, lyric: &Lyric) -> Result<bool> {
        let params = vec![
            Value::Text(lyric.title.clone()),
            Value::Text(Parts::from(lyric.parts.clone()).to_text()),
            Value::Text(lyric.id.clone()),
        ];
        self.0
            .execute(sql::SQL_UPDATE_LYRIC, &params)
            .map(|c| c > 0)
    }

    pub fn insert_lyric(&self, lyric: &Lyric) -> Result<()> {
        let params = vec![
            Value::Text(lyric.id.clone()),
            Value::Text(lyric.title.clone()),
            Value::Text(Parts::from(lyric.parts.clone()).to_text()),
            Value::Text(Uuid::default().to_string()),
        ];
        self.0.execute(sql::SQL_INSERT_LYRIC, &params).map_to_unit()
    }

    fn get_playlist_members<D>(&self, playlist_id: D) -> Result<Vec<String>>
    where
        D: Display,
    {
        self.0
            .query::<LyricId>(
                sql::SQL_GET_MEMBER_LYRICS,
                &[Value::Text(playlist_id.to_string())],
            )
            .map(|id_list| id_list.into_iter().map(|lyric_id| lyric_id.id()).collect())
    }

    pub fn get_playlist_list(&self) -> Result<Vec<Playlist>> {
        let mut playlists = self.0.query::<Playlist>(sql::SQL_GET_PLAYLIST_LIST, &[])?;

        for playlist in playlists.iter_mut() {
            playlist.members = self.get_playlist_members(playlist.id.clone())?;
        }
        Ok(playlists)
    }

    pub fn get_playlist<D>(&self, id: D) -> Result<Option<Playlist>>
    where
        D: Display,
    {
        let params = vec![Value::Text(id.to_string())];
        let result = self
            .0
            .query::<Playlist>(sql::SQL_GET_PLAYLIST, &params)
            .map_first()?;
        match result {
            Some(mut playlist) => {
                playlist.members = self.get_playlist_members(&playlist.id)?;
                Ok(Some(playlist.clone()))
            }
            None => Ok(None),
        }
    }

    pub fn delete_playlist<D>(&self, id: D) -> Result<()>
    where
        D: Display,
    {
        self.0
            .execute(sql::SQL_DELETE_PLAYLIST, &[Value::Text(id.to_string())])
            .map_to_unit()
    }

    pub fn delete_members(&self, playlist_id: &str) -> Result<i64> {
        self.0
            .execute(sql::SQL_DELETE_MEMBER, &[Value::Text(playlist_id.into())])
    }

    pub fn insert_members(&self, playlist_id: &str, lyric_ids: &[String]) -> Result<()> {
        lyric_ids
            .iter()
            .enumerate()
            .map(|(i, s)| (i + 1, s))
            .map(|(i, lyric_id)| {
                self.0.execute(
                    sql::SQL_INSERT_MEMBER,
                    &[
                        Value::Text(playlist_id.into()),
                        Value::Text(lyric_id.clone()),
                        Value::Integer(i.try_into().unwrap()),
                    ],
                )
            })
            .collect::<Result<Vec<_>>>()
            .map_to_unit()
    }

    pub fn update_playlist(&self, playlist: &Playlist) -> Result<()> {
        self.begin_transaction()?;
        self.delete_members(&playlist.id)
            .and_then(|_| {
                self.0.execute(
                    sql::SQL_UPDATE_PLAYLIST,
                    &[
                        Value::Text(playlist.title.clone()),
                        Value::Text(Uuid::default().to_string()),
                        Value::Text(playlist.id.clone()),
                    ],
                )
            })
            .and_then(|_| self.insert_members(&playlist.id, &playlist.members))
            .and_then(|_| self.commit())
            .inspect_err(|error| {
                if self.roll_back().is_err() {
                    message::rollback_failure(error);
                }
            })
    }

    pub fn insert_playlist(&self, playlist: &Playlist, transact: bool) -> Result<()> {
        if transact {
            self.begin_transaction()?;
        }

        self.0
            .execute(
                sql::SQL_INSERT_PLAYLIST,
                &[
                    Value::Text(playlist.id.clone()),
                    Value::Text(playlist.title.clone()),
                    Value::Text(Uuid::default().to_string()),
                ],
            )
            .and_then(|_| self.insert_members(&playlist.id, &playlist.members))
            .and_then(|_| if transact { self.commit() } else { Ok(()) })
            .inspect_err(|error| {
                if transact && self.roll_back().is_err() {
                    message::rollback_failure(error);
                }
            })
    }

    pub fn update_lyric_list_etag(&self) -> Result<()> {
        self.0
            .execute(
                sql::SQL_UPDATE_LYRIC_LIST_ETAG,
                &[Value::Text(Uuid::default().to_string())],
            )
            .map_to_unit()
    }

    pub fn update_playlist_list_etag(&self) -> Result<()> {
        self.0
            .execute(
                sql::SQL_UPDATE_PLAYLIST_LIST_ETAG,
                &[Value::Text(Uuid::default().to_string())],
            )
            .map_to_unit()
    }

    pub fn delete_all_playlists(&self) -> Result<()> {
        self.0
            .execute(sql::SQL_DELETE_ALL_PLAYLISTS, &[])
            .map_to_unit()
    }

    pub fn delete_all_lyrics(&self) -> Result<()> {
        self.0
            .execute(sql::SQL_DELETE_ALL_LYRICS, &[])
            .map_to_unit()
    }

    pub fn delete_all_members(&self) -> Result<()> {
        self.0
            .execute(sql::SQL_DELETE_ALL_MEMBERS, &[])
            .map_to_unit()
    }

    pub fn replace_db(&self, db: &Db) -> Result<()> {
        self.begin_transaction()?;

        self.delete_all_playlists()
            .and_then(|_| self.delete_all_lyrics())
            .and_then(|_| self.delete_all_members())
            .and_then(|_| {
                db.lyrics
                    .iter()
                    .map(|lyric| self.insert_lyric(lyric))
                    .collect::<Result<Vec<_>>>()
            })
            .and_then(|_| {
                db.playlists
                    .iter()
                    .map(|playlist| self.insert_playlist(playlist, false))
                    .collect::<Result<Vec<_>>>()
            })
            .and_then(|_| self.update_lyric_list_etag())
            .and_then(|_| self.update_playlist_list_etag())
            .and_then(|_| self.commit())
            .inspect_err(|error| {
                if self.roll_back().is_err() {
                    message::rollback_failure(error);
                }
            })
    }
}

mod sql {
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

    pub const SQL_UPDATE_LYRIC_LIST_ETAG: &str =
        "UPDATE list_etag SET etag = ? WHERE id = 'lyrics'";
    pub const SQL_UPDATE_PLAYLIST_LIST_ETAG: &str =
        "UPDATE list_etag SET etag = ? WHERE id = 'playlists'";

    pub const SQL_DELETE_ALL_PLAYLISTS: &str = "DELETE FROM playlist";
    pub const SQL_DELETE_ALL_LYRICS: &str = "DELETE FROM lyric";
    pub const SQL_DELETE_ALL_MEMBERS: &str = "DELETE FROM member";
}

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use model::{error::Error, Db, Lyric, Playlist, TryFromJson, Uuid};
    use spin_sdk::sqlite::Row;

    use super::Connection;

    const MIGRATIONS: &str = include_str!("../migrations.sql");

    fn open_database() -> super::Connection {
        Connection::try_open_default(Some(MIGRATIONS)).unwrap()
    }

    #[test]
    fn test_open_database() {
        open_database();
    }

    #[test]
    fn insert_lyric() {
        let connection = open_database();

        let id = Uuid::default().to_string();
        let mut lyric = Lyric::new(id.clone(), "Zie maar hoe je het doet".to_owned(), vec![]);
        connection.insert_lyric(&lyric).unwrap();

        let stored_lyric = connection.get_lyric(id.clone()).unwrap().unwrap();
        assert!(stored_lyric.created.is_some());
        assert!(stored_lyric.modified.is_some());
        assert!(stored_lyric.etag.is_some());

        "Hallo allemaal".clone_into(&mut lyric.title);
        thread::sleep(Duration::from_millis(5));
        if connection.update_lyric(&lyric).unwrap() {
            let lyric = connection.get_lyric(id.clone()).unwrap().unwrap();
            println!("created: {}", lyric.created.unwrap());
            println!("modified: {}", lyric.modified.unwrap());
        };
    }

    #[test]
    fn insert_playlist() {
        let connection = open_database();

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
        connection.insert_playlist(&playlist, false).unwrap();

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
        let connection = open_database();

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
            .0
            .query::<Table>(
                "SELECT name FROM sqlite_schema WHERE type ='table' AND  name NOT LIKE 'sqlite_%';",
                &[],
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
        let connection = open_database();

        let db = Db::try_from_json(include_bytes!("../data/db.json")).unwrap();
        connection.replace_db(&db).unwrap();

        let lyrics = connection.get_lyric_list().unwrap();
        assert_eq!(lyrics.len(), 57);

        let playlists = connection.get_playlist_list().unwrap();
        assert_eq!(playlists.len(), 1);
    }
}
