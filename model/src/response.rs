use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use spin_sdk::{
    http::{HeaderMap, IntoResponse, StatusCode},
    wasip3::http_compat::http_into_wasi_response,
};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{Etag, error::Error};

pub fn unauthenticated() -> wasip3::http_compat::Response<String> {
    wasip3::http_compat::Response::builder()
        .status(401)
        .header("WWW-Authenticate", "Basic realm=\"Lipl Api\"")
        .body(String::new())
        .unwrap()
}

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
            Self::NotFound => StatusCode::NOT_FOUND.into_response(),
            Self::Authentication(_) => http_into_wasi_response(unauthenticated()),
            Self::Body => StatusCode::BAD_REQUEST.into_response(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
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
