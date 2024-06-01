use model::basic_authentication::Authentication;
use model::error::{AuthenticationError, Error};
use spin_sdk::http::{IntoResponse, Params, Request, Response};

use model::response::{
    bad_request, created, if_none_match, no_content, not_found, not_modified, JsonResponse,
};
use model::{Db, Etag, Lyric, LyricPost, Playlist, PlaylistPost, TryFromJson, Uuid};

use crate::{persistence::Connection, Result};

fn connect_user(req: &Request) -> Result<Connection> {
    let connection = Connection::try_open_default(None)?;
    let authorization_value = req
        .header("Authorization")
        .ok_or(AuthenticationError::AuthenticationHeader)?;
    let authorization_s = authorization_value
        .as_str()
        .ok_or(AuthenticationError::AuthenticationHeader)?;
    let authentication = authorization_s
        .parse::<Authentication>()
        .map_err(|_| AuthenticationError::AuthenticationHeader)?;
    match authentication {
        Authentication::Basic(credentials) => {
            connection
                .is_valid_user(&credentials.username, &credentials.password)
                .map_err(|_| AuthenticationError::Username)?;
        }
    }
    Ok(connection)
}

pub fn get_lyric_list(req: Request, _: Params) -> Result<Response> {
    let lyrics = connect_user(&req).and_then(|c| c.get_lyric_list())?;
    if Some(lyrics.etag()) == if_none_match(&req) {
        Ok(not_modified())
    } else {
        Ok(JsonResponse::new(lyrics, req).into_response())
    }
}

pub fn get_lyric(req: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(not_found());
    };

    match connect_user(&req).and_then(|c| c.get_lyric(id))? {
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

pub fn insert_lyric(req: Request, _: Params) -> Result<impl IntoResponse> {
    let Ok(lyric) = Lyric::try_from_json(req.body()) else {
        return Ok(bad_request());
    };
    connect_user(&req)
        .and_then(|c| c.insert_lyric(&lyric))
        .map(|_| created())
}

pub fn update_lyric(req: Request, params: Params) -> Result<impl IntoResponse> {
    let id = params.get("id").ok_or(Error::NotFound)?;
    let lyric_post = LyricPost::try_from_json(req.body()).map_err(|_| Error::Body)?;
    let lyric = Lyric::new(
        id.to_owned(),
        lyric_post.title.clone(),
        lyric_post.parts.clone(),
    );
    connect_user(&req)
        .and_then(|c| c.update_lyric(&lyric))
        .map(|_| no_content())
}

pub fn delete_lyric(req: Request, params: Params) -> Result<impl IntoResponse> {
    let id = params.get("id").ok_or(Error::NotFound)?;
    connect_user(&req)
        .and_then(|c| c.delete_lyric(id))
        .map(|_| no_content())
}

pub fn get_playlist_list(req: Request, _: Params) -> Result<impl IntoResponse> {
    let playlists = connect_user(&req).and_then(|c| c.get_playlist_list())?;
    if Some(playlists.etag()) == if_none_match(&req) {
        Ok(not_modified())
    } else {
        Ok(JsonResponse::new(playlists, req).into_response())
    }
}

pub fn get_playlist(req: Request, params: Params) -> Result<impl IntoResponse> {
    let id = params.get("id").ok_or(Error::NotFound)?;
    match connect_user(&req).and_then(|c| c.get_playlist(id))? {
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

pub fn insert_playlist(req: Request, _: Params) -> Result<impl IntoResponse> {
    let playlist = Playlist::try_from_json(req.body()).map_err(|_| Error::Body)?;
    connect_user(&req)
        .and_then(|c| c.insert_playlist(&playlist, true))
        .map(|_| created())
}

pub fn update_playlist(req: Request, params: Params) -> Result<impl IntoResponse> {
    let id = params.get("id").ok_or(Error::NotFound)?;
    let playlist_post = PlaylistPost::try_from_json(req.body()).map_err(|_| Error::Body)?;
    let playlist = Playlist::new(
        id.to_owned(),
        playlist_post.title.clone(),
        playlist_post.members.clone(),
    );
    connect_user(&req)
        .and_then(|c| c.update_playlist(&playlist))
        .map(|_| no_content())
}

pub fn delete_playlist(req: Request, params: Params) -> Result<impl IntoResponse> {
    let id = params.get("id").ok_or(Error::NotFound)?;
    connect_user(&req)
        .and_then(|c| c.delete_playlist(id))
        .map(|_| no_content())
}

pub fn replace_db(req: Request, _: Params) -> Result<impl IntoResponse> {
    let db = Db::try_from_json(req.body())?;
    let connection = connect_user(&req)?;
    connection.replace_db(&db)?;

    Ok(no_content())
}

pub fn get_db(req: Request, _: Params) -> Result<impl IntoResponse> {
    let connection = connect_user(&req)?;
    let lyrics = connection.get_lyric_list()?;
    let playlists = connection.get_playlist_list()?;
    let db = Db { lyrics, playlists };
    Ok(JsonResponse::new(db, req))
}

pub fn get_uuid(req: Request, params: Params) -> Result<impl IntoResponse> {
    let uuid_str = params.get("id").ok_or(Error::NotFound)?;
    let uuid = Uuid::from_uuid_str(uuid_str)?;

    Ok(JsonResponse::new(uuid.to_string(), req))
}
