use std::sync::OnceLock;

use crate::api::Api;
use model::{
    basic_authentication::{unauthenticated, Authentication},
    error::Error, Uuid,
};
use spin_sdk::{
    http::{IntoResponse, Request, Response},
    http_component,
    variables::get,
};

mod api;
mod handler;
mod message;
mod persistence;

type Result<T> = std::result::Result<T, Error>;

fn request_id() -> &'static Uuid {
    static REQUEST_ID: OnceLock<Uuid> = OnceLock::new();
    REQUEST_ID.get_or_init(Uuid::default)
}


/// A simple Spin HTTP component.
#[http_component]
fn handle_lipl_storage_spin(req: Request) -> Result<Response> {
    message::request_received(req.path());
    let api = Api::default();
    if let Some(authorization_value) = req
        .header("Authorization")
        .and_then(|header| header.as_str())
    {
        let authentication = authorization_value.parse::<Authentication>()?;
        if authentication.is_valid_user(get("lipl_username")?, get("lipl_password")?) {
            api.handle(req).map(|r| r.into_response())
        } else {
            Ok(unauthenticated())
        }
    } else {
        Ok(unauthenticated())
    }
}
