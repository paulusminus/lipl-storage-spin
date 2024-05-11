CREATE TABLE IF NOT EXISTS lyric (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    parts TEXT NOT NULL,
    created TEXT NOT NULL,
    modified TEXT NOT NULL,
    etag TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created TEXT NOT NULL,
    modified TEXT NOT NULL,
    etag TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS member(
    lyric_id TEXT NOT NULL REFERENCES lyric(id) ON DELETE CASCADE,
    playlist_id TEXT NOT NULL REFERENCES playlist(id) ON DELETE CASCADE,
    ordering INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS list_etag(
    id TEXT NOT NULL PRIMARY KEY,
    etag TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS member_lyric_playlist on member (lyric_id, playlist_id, ordering);
CREATE UNIQUE INDEX IF NOT EXISTS lyric_title on lyric (title);
CREATE UNIQUE INDEX IF NOT EXISTS playlist_title on playlist (title);
CREATE UNIQUE INDEX IF NOT EXISTS lyric_etag on lyric (etag);
CREATE UNIQUE INDEX IF NOT EXISTS playlist_etag on playlist (etag);
CREATE UNIQUE INDEX IF NOT EXISTS list_etags on list_etag (etag);

INSERT INTO list_etag (id, etag) VALUES ("lyrics", "8EDYXnTEey7cMSSsRd4EE8");
INSERT INTO list_etag (id, etag) VALUES ("playlists", "Qr4kJo6LoiKGDMtvfbUoP3");