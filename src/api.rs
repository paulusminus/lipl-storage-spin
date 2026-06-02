use crate::handler;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use tower_http::auth::AddAuthorizationLayer;

pub fn create_router(username: &str, password: &str) -> Router {
    Router::new()
        .route("/lipl/api/v1/lyric", get(handler::get_lyric_list))
        .route("/lipl/api/v1/lyric/{id}", get(handler::get_lyric))
        .route("/lipl/api/v1/lyric", post(handler::insert_lyric))
        .route("/lipl/api/v1/lyric/{id}", put(handler::update_lyric))
        .route("/lipl/api/v1/lyric/{id}", delete(handler::delete_lyric))
        .route("/lipl/api/v1/playlist", get(handler::get_playlist_list))
        .route("/lipl/api/v1/playlist/{id}", get(handler::get_playlist))
        .route("/lipl/api/v1/playlist", post(handler::insert_playlist))
        .route("/lipl/api/v1/playlist/{id}", put(handler::update_playlist))
        .route(
            "/lipl/api/v1/playlist/{id}",
            delete(handler::delete_playlist),
        )
        .route("/lipl/api/v1/db", get(handler::get_db))
        .route("/lipl/api/v1/db", post(handler::replace_db))
        .route("/lipl/api/v1/uuid/{id}", get(handler::get_uuid))
        .route("/lipl/api/v1/user", get(handler::get_user_list))
        .layer(AddAuthorizationLayer::basic(username, password))
}
