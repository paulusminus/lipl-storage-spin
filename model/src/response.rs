use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use serde::Serialize;
use spin_sdk::http::{IntoResponse, Request, Response};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{basic_authentication::unauthenticated, convert::ToJson, error::Error, Etag};

const NOT_MODIFIED: u16 = 304;
const NOT_FOUND: u16 = 404;
const BAD_REQUEST: u16 = 400;
const NO_CONTENT: u16 = 204;
const CREATED: u16 = 201;
const INTERNAL_SERVER_ERROR: u16 = 500;

macro_rules! status {
    ($name:ident, $code:expr) => {
        pub fn $name() -> Response {
            Response::new($code, ())
        }
    };
}

status!(not_modified, NOT_MODIFIED);
status!(not_found, NOT_FOUND);
status!(bad_request, BAD_REQUEST);
status!(no_content, NO_CONTENT);
status!(created, CREATED);
status!(internal_server_error, INTERNAL_SERVER_ERROR);

pub fn if_none_match(req: &Request) -> Option<String> {
    req.header("If-None-Match")
        .and_then(|h| h.as_str())
        .map(String::from)
}

impl IntoResponse for Error {
    fn into_response(self) -> spin_sdk::http::Response {
        eprintln!("Error: {}", &self);
        match self {
            Self::NotFound => not_found(),
            Self::Authentication(_) => unauthenticated(),
            Self::Body => bad_request(),
            _ => internal_server_error(),
        }
    }
}

pub struct JsonResponse<S: Serialize> {
    s: S,
    #[allow(dead_code)]
    request: Request,
}

impl<S: Serialize + Etag> JsonResponse<S> {
    pub fn new(s: S, request: Request) -> Self {
        Self { s, request }
    }
}

impl<S: Serialize + Etag> IntoResponse for JsonResponse<S> {
    fn into_response(self) -> Response {
        let body = self.s.to_json().unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .header("ETag", self.s.etag())
            .body(body)
            .build()
    }
}

impl<T: Hash> Etag for T {
    fn etag(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        let bytes = hash.to_le_bytes();
        STANDARD_NO_PAD.encode(bytes)
    }
}
