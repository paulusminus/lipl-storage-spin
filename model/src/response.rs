use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use serde::Serialize;
use spin_sdk::{
    http::{HeaderMap, IntoResponse, Response, StatusCode},
    wasip3::{http::types::ErrorCode, http_compat::http_into_wasi_response},
};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{Etag, basic_authentication::unauthenticated, convert::ToJson, error::Error};

const NOT_MODIFIED: u16 = 304;
const NOT_FOUND: u16 = 404;
const BAD_REQUEST: u16 = 400;
const NO_CONTENT: u16 = 204;
const CREATED: u16 = 201;
const INTERNAL_SERVER_ERROR: u16 = 500;

macro_rules! status {
    ($name:ident, $code:expr) => {
        pub fn $name() -> spin_sdk::wasip3::http::types::Response {
            StatusCode::from_u16($code)
                .unwrap()
                .into_response()
                .unwrap()
        }
    };
}

status!(not_modified, NOT_MODIFIED);
status!(not_found, NOT_FOUND);
status!(bad_request, BAD_REQUEST);
status!(no_content, NO_CONTENT);
status!(created, CREATED);
status!(internal_server_error, INTERNAL_SERVER_ERROR);

pub fn if_none_match(headers: &HeaderMap) -> Option<String> {
    headers
        .get("If-None-Match")
        .and_then(|h| h.to_str().ok())
        .map(String::from)
}

impl IntoResponse for Error {
    fn into_response(
        self,
    ) -> std::result::Result<
        spin_sdk::wasip3::http::types::Response,
        spin_sdk::wasip3::http::types::ErrorCode,
    > {
        eprintln!("Error: {}", &self);
        match self {
            Self::NotFound => Ok(not_found()),
            Self::Authentication(_) => Ok(unauthenticated()),
            Self::Body => Ok(bad_request()),
            _ => Ok(internal_server_error()),
        }
    }
}

pub struct JsonResponse<S: Serialize> {
    s: S,
    #[allow(dead_code)]
    headers: HeaderMap,
}

impl<S: Serialize + Etag> JsonResponse<S> {
    pub fn new(s: S, headers: HeaderMap) -> Self {
        Self { s, headers }
    }
}

impl<S: Serialize + Etag> IntoResponse for JsonResponse<S> {
    fn into_response(
        self,
    ) -> std::result::Result<spin_sdk::wasip3::http::types::Response, ErrorCode> {
        let body = self.s.to_json().unwrap();

        let response = Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .header("ETag", self.s.etag())
            .body(body)
            .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;

        http_into_wasi_response(response)
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
