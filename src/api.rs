use std::fmt::Display;

use spin_sdk::{
    http::{IntoResponse, Request, Router},
    http_router,
};

use crate::{handler, Result};

pub struct Api {
    router: Router,
}

impl Api {
    pub(crate) fn handle(&self, req: Request) -> Result<impl IntoResponse> {
        Ok(self.router.handle(req))
    }
}

impl Display for Api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.router)
    }
}

impl Default for Api {
    fn default() -> Self {
        let router = http_router!(
            GET "/lipl/api/v1/lyric" => handler::get_lyric_list,
            GET "/lipl/api/v1/lyric/:id" => handler::get_lyric,
            POST "/lipl/api/v1/lyric" => handler::insert_lyric,
            PUT "/lipl/api/v1/lyric/:id" => handler::update_lyric,
            DELETE "/lipl/api/v1/lyric/:id" => handler::delete_lyric,
            GET "/lipl/api/v1/playlist" => handler::get_playlist_list,
            GET "/lipl/api/v1/playlist/:id" => handler::get_playlist,
            POST "/lipl/api/v1/playlist" => handler::insert_playlist,
            PUT "/lipl/api/v1/playlist/:id" => handler::update_playlist,
            DELETE "/lipl/api/v1/playlist/:id" => handler::delete_playlist,
            GET "/lipl/api/v1/db" => handler::get_db,
            POST "/lipl/api/v1/db" => handler::replace_db,
            GET "/lipl/api/v1/uuid/:id" => handler::get_uuid
        );
        Self { router }
    }
}
