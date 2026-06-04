use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use model::response::if_none_match;
use model::{Db, Etag, Lyric, LyricPost, Playlist, Uuid};

use crate::{Result, persistence::Connection};

pub async fn get_lyric_list(headers: HeaderMap) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    let lyrics = connection.select_lyric().await?;
    if Some(lyrics.etag()) == if_none_match(&headers) {
        Ok(StatusCode::NOT_MODIFIED.into_response())
    } else {
        Ok(Json(lyrics).into_response())
    }
}

pub async fn get_lyric(headers: HeaderMap, Path(id): Path<String>) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    match connection.select_lyric_by_id(&id).await? {
        Some(lyric) => {
            if Some(lyric.etag()) == if_none_match(&headers) {
                Ok(StatusCode::NOT_MODIFIED.into_response())
            } else {
                Ok(Json(lyric).into_response())
            }
        }
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

pub async fn insert_lyric(Json(lyric): Json<Lyric>) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    connection
        .insert_lyric(&lyric)
        .await
        .map(|_| StatusCode::CREATED.into_response())
}

pub async fn update_lyric(
    Path(id): Path<String>,
    Json(lyric_post): Json<LyricPost>,
) -> Result<impl IntoResponse> {
    let lyric = Lyric::new(
        id.to_owned(),
        lyric_post.title.clone(),
        lyric_post.parts.clone(),
    );
    let connection = Connection::try_open_default(None).await?;
    connection
        .update_lyric(&lyric)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn delete_lyric(Path(id): Path<String>) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    connection
        .delete_lyric(&id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn get_playlist_list(headers: HeaderMap) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    let playlists = connection.select_playlist().await?;
    if Some(playlists.etag()) == if_none_match(&headers) {
        Ok(StatusCode::NOT_MODIFIED.into_response())
    } else {
        Ok(Json(playlists).into_response())
    }
}

pub async fn get_playlist(Path(id): Path<String>, headers: HeaderMap) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    match connection.select_playlist_by_id(&id).await? {
        Some(playlist) => {
            if Some(playlist.etag()) == if_none_match(&headers) {
                Ok(StatusCode::NOT_MODIFIED.into_response())
            } else {
                Ok(Json(playlist).into_response())
            }
        }
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

pub async fn insert_playlist(Json(playlist): Json<Playlist>) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    connection
        .insert_playlist(&playlist, true)
        .await
        .map(|_| StatusCode::CREATED)
}

pub async fn update_playlist(
    Path(_): Path<String>,
    Json(playlist): Json<Playlist>,
) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    connection
        .update_playlist(&playlist)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn delete_playlist(Path(id): Path<String>) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    connection
        .delete_playlist_by_id(&id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn replace_db(Json(db): Json<Db>) -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    connection
        .replace_db(&db)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn get_db() -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(None).await?;
    let lyrics = connection.select_lyric().await?;
    let playlists = connection.select_playlist().await?;
    let db = Db { lyrics, playlists };
    Ok(Json(db))
}

pub async fn get_uuid(Path(id): Path<String>) -> Result<impl IntoResponse> {
    let uuid = Uuid::from_uuid_str(&id)?;
    Ok(Json(uuid.to_string()))
}

pub async fn get_user_list() -> Result<impl IntoResponse> {
    let connection = Connection::try_open_default(Some(include_str!("../migrations.sql"))).await?;
    connection.select_user().await.map(Json)
}
