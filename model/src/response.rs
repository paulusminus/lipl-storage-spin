use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use spin_sdk::http::{HeaderMap, IntoResponse, StatusCode};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{Etag, basic_authentication::unauthenticated, error::Error};

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
            Self::Authentication(_) => Ok(unauthenticated()),
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
