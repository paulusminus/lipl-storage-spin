use std::{sync::OnceLock, time::Instant};

use crate::api::Api;
use model::{error::Error, response::no_content, Uuid};
use spin_sdk::{
    http::{IntoResponse, Request, Response},
    http_component,
};

mod api;
pub mod handler;
mod message;
pub mod persistence;

type Result<T> = std::result::Result<T, Error>;

fn request_id() -> &'static Uuid {
    static REQUEST_ID: OnceLock<Uuid> = OnceLock::new();
    REQUEST_ID.get_or_init(Uuid::default)
}

fn now() -> &'static Instant {
    static NOW: OnceLock<Instant> = OnceLock::new();
    NOW.get_or_init(Instant::now)
}

fn header_value<'a>(req: &'a Request, name: &'a str) -> Option<&'a str> {
    req.headers()
        .find(|header| header.0.to_lowercase() == name)
        .and_then(|header| header.1.as_str())
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_lipl_storage_spin(req: Request) -> Result<Response> {
    message::request_received(req.path(), req.method());
    if let Some(referer) = header_value(&req, "referer") {
        message::dump_header("referer", referer);
    }
    if let Some(referer) = header_value(&req, "host") {
        message::dump_header("host", referer);
    }

    if req.path() == "/lipl/api/v1/health" {
        return Ok(no_content());
    }

    let api = Api::default();
    api.handle(req)
        .map(|r| r.into_response())
        .inspect_err(|error| {
            eprintln!(
                "{}: Error {} after {} milliseconds",
                request_id(),
                error,
                now().elapsed().as_millis()
            );
        })
        .inspect(|x| {
            println!(
                "{}: Success {} after {} milliseconds",
                request_id(),
                x.status(),
                now().elapsed().as_millis()
            );
        })
}
