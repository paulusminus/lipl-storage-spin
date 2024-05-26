use model::error::Error;
use spin_sdk::http::{IntoResponse, Params, Request, Response};

use model::response::{
    bad_request, created, if_none_match, no_content, not_found, not_modified, JsonResponse,
};
use model::{Db, Etag, Lyric, LyricPost, Playlist, PlaylistPost, TryFromJson, Uuid};

use crate::{persistence::Connection, Result};

pub(crate) fn get_lyric_list(req: Request, _: Params) -> Result<Response> {
    let lyrics = Connection::try_open_default(None).and_then(|c| c.get_lyric_list())?;
    if Some(lyrics.etag()) == if_none_match(&req) {
        Ok(not_modified())
    } else {
        Ok(JsonResponse::new(lyrics, req).into_response())
    }
}

pub(crate) fn get_lyric(req: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(not_found());
    };

    match Connection::try_open_default(None).and_then(|c| c.get_lyric(id))? {
        Some(lyric) => {
            if Some(lyric.etag()) == if_none_match(&req) {
                Ok(not_modified())
            } else {
                Ok(JsonResponse::new(lyric, req).into_response())
            }
        }
        None => Ok(not_found()),
    }
}

pub(crate) fn insert_lyric(req: Request, _: Params) -> Result<impl IntoResponse> {
    let Ok(lyric) = Lyric::try_from_json(req.body()) else {
        return Ok(bad_request());
    };
    Connection::try_open_default(None)
        .and_then(|c| c.insert_lyric(&lyric))
        .map(|_| created())
}

pub(crate) fn update_lyric(req: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(not_found());
    };
    let Ok(lyric_post) = LyricPost::try_from_json(req.body()) else {
        return Ok(bad_request());
    };
    let lyric = Lyric::new(
        id.to_owned(),
        lyric_post.title.clone(),
        lyric_post.parts.clone(),
    );
    Connection::try_open_default(None).and_then(|c| c.update_lyric(&lyric))
    .map(|_| no_content())
}

pub(crate) fn delete_lyric(_: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(Response::new(400, ()));
    };
    Connection::try_open_default(None).and_then(|c| c.delete_lyric(id))
    .map(|_| no_content())
}

pub(crate) fn get_playlist_list(req: Request, _: Params) -> Result<impl IntoResponse> {
    let playlists = Connection::try_open_default(None).and_then(|c| c.get_playlist_list())?;
    if Some(playlists.etag()) == if_none_match(&req) {
        Ok(not_modified())
    } else {
        Ok(JsonResponse::new(playlists, req).into_response())
    }
}

pub(crate) fn get_playlist(req: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(not_found());
    };
    match Connection::try_open_default(None).and_then(|c| c.get_playlist(id))? {
        Some(playlist) => {
            if Some(playlist.etag()) == if_none_match(&req) {
                Ok(not_modified())
            } else {
                Ok(JsonResponse::new(playlist, req).into_response())
            }
        }
        None => Ok(not_found()),
    }
}

pub(crate) fn insert_playlist(req: Request, _: Params) -> Result<impl IntoResponse> {
    let Ok(playlist) = Playlist::try_from_json(req.body()) else {
        return Ok(bad_request());
    };
    Connection::try_open_default(None)
        .and_then(|c| c.insert_playlist(&playlist, true))
        .map(|_| created())
}

pub(crate) fn update_playlist(req: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(not_found());
    };
    let Ok(playlist_post) = PlaylistPost::try_from_json(req.body()) else {
        return Ok(bad_request());
    };
    let playlist = Playlist::new(
        id.to_owned(),
        playlist_post.title.clone(),
        playlist_post.members.clone(),
    );
    Connection::try_open_default(None)
        .and_then(|c| c.update_playlist(&playlist))
        .map(|_| no_content())
}

pub(crate) fn delete_playlist(_: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(not_found());
    };
    Connection::try_open_default(None).and_then(|c| c.delete_playlist(id))
    .map(|_| no_content())
}

pub(crate) fn replace_db(req: Request, _: Params) -> Result<impl IntoResponse> {
    let db = Db::try_from_json(req.body())?;
    let connection = Connection::try_open_default(None)?;
    connection.replace_db(&db)?;

    Ok(no_content())
}

pub(crate) fn get_db(req: Request, _: Params) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None)?;
    let lyrics = connection.get_lyric_list()?;
    let playlists = connection.get_playlist_list()?;
    let db = Db { lyrics, playlists };
    Ok(JsonResponse::new(db, req))
}

pub(crate) fn get_uuid(req: Request, params: Params) -> Result<impl IntoResponse> {
    let uuid_str = params.get("id").ok_or(Error::NotFound)?;
    let uuid = Uuid::from_uuid_str(uuid_str)?;

    Ok(JsonResponse::new(uuid.to_string(), req))
}
