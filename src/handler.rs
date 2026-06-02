use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use model::basic_authentication::Authentication;
use model::error::AuthenticationError;
use model::response::if_none_match;
use model::{Db, Etag, Lyric, LyricPost, Playlist, Uuid};

use crate::{Result, persistence::Connection};

async fn connect_user(headers: &HeaderMap) -> Result<Connection> {
    let connection = Connection::try_open_default(None).await?;
    let authorization_value = headers
        .get("Authorization")
        .ok_or(AuthenticationError::AuthenticationHeader)?;
    let authorization_s = authorization_value
        .to_str()
        .map_err(|_| AuthenticationError::AuthenticationHeader)?;
    let authentication = authorization_s
        .parse::<Authentication>()
        .map_err(|_| AuthenticationError::AuthenticationHeader)?;
    match authentication {
        Authentication::Basic(credentials) => {
            connection
                .is_valid_user(&credentials.username, &credentials.password)
                .await
                .map_err(|_| AuthenticationError::Username)?;
        }
    }
    Ok(connection)
}

pub async fn get_lyric_list(headers: HeaderMap) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    let lyrics = connection.select_lyric().await?;
    if Some(lyrics.etag()) == if_none_match(&headers) {
        Ok(StatusCode::NOT_MODIFIED.into_response())
    } else {
        Ok(Json(lyrics).into_response())
    }
}

pub async fn get_lyric(headers: HeaderMap, Path(id): Path<String>) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
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

pub async fn insert_lyric(
    headers: HeaderMap,
    Json(lyric): Json<Lyric>,
) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    connection
        .insert_lyric(&lyric)
        .await
        .map(|_| StatusCode::CREATED.into_response())
}

pub async fn update_lyric(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(lyric_post): Json<LyricPost>,
) -> Result<impl IntoResponse> {
    let lyric = Lyric::new(
        id.to_owned(),
        lyric_post.title.clone(),
        lyric_post.parts.clone(),
    );
    let connection = connect_user(&headers).await?;
    connection
        .update_lyric(&lyric)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn delete_lyric(Path(id): Path<String>, headers: HeaderMap) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    connection
        .delete_lyric(&id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn get_playlist_list(headers: HeaderMap) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    let playlists = connection.select_playlist().await?;
    if Some(playlists.etag()) == if_none_match(&headers) {
        Ok(StatusCode::NOT_MODIFIED.into_response())
    } else {
        Ok(Json(playlists).into_response())
    }
}

pub async fn get_playlist(Path(id): Path<String>, headers: HeaderMap) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
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

pub async fn insert_playlist(
    headers: HeaderMap,
    Json(playlist): Json<Playlist>,
) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    connection
        .insert_playlist(&playlist, true)
        .await
        .map(|_| StatusCode::CREATED)
}

pub async fn update_playlist(
    headers: HeaderMap,
    Path(_): Path<String>,
    Json(playlist): Json<Playlist>,
) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    connection
        .update_playlist(&playlist)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn delete_playlist(
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    connection
        .delete_playlist_by_id(&id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn replace_db(headers: HeaderMap, Json(db): Json<Db>) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    connection.replace_db(&db).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_db(headers: HeaderMap) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    let lyrics = connection.select_lyric().await?;
    let playlists = connection.select_playlist().await?;
    let db = Db { lyrics, playlists };
    Ok(Json(db))
}

pub async fn get_uuid(Path(id): Path<String>) -> Result<impl IntoResponse> {
    let uuid = Uuid::from_uuid_str(&id)?;
    Ok(Json(uuid.to_string()))
}

pub async fn get_user_list(headers: HeaderMap) -> Result<impl IntoResponse> {
    let connection = connect_user(&headers).await?;
    let users = connection.select_user().await?;
    Ok(Json(users))
}
