use std::{num::ParseIntError, str::Utf8Error};

use spin_sdk::http::IntoResponse;

use crate::{
    basic_authentication::unauthenticated,
    response::{bad_request, internal_server_error, not_found},
};

#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Username")]
    Username,

    #[error("Password")]
    Password,

    #[error("Authentication header")]
    AuthenticationHeader,

    #[error("Unsupported")]
    Unsupported,

    #[error("Decode base64")]
    DecodeBase64(#[from] base64::DecodeError),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Invalid body")]
    Body,

    #[error("Utf 8 encoding")]
    Utf8(#[from] Utf8Error),

    #[error("Json")]
    Json(#[from] serde_json::Error),

    #[error("Column {0}")]
    Column(String),

    #[error("Chrono: {0}")]
    Chrono(#[from] chrono::ParseError),

    #[error("IO: {0}")]
    IO(#[from] std::io::Error),

    #[error("Base58 decode: {0}")]
    Base58Decode(#[from] bs58::decode::Error),

    #[error("Uuid: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Variable: {0}")]
    Variable(#[from] spin_sdk::variables::Error),

    #[error("Missing lyric id")]
    MissingLyricId,

    #[error("Missing column: {0}")]
    MissingColumn(&'static str),

    #[error("Authentication")]
    Authentication(#[from] AuthenticationError),

    #[error("Parsing int: {0}")]
    ParseInt(#[from] ParseIntError),

    #[cfg(target_family = "wasm")]
    #[error("Spin SQLite: {0}")]
    SpinSQLite(#[from] spin_sdk::sqlite::Error),

    #[cfg(not(target_family = "wasm"))]
    #[error("Rusqlite: {0}")]
    Rusqlite(#[from] rusqlite::Error),
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

pub trait ErrInto<T, E> {
    fn err_into(self) -> Result<T, Error>;
}

impl<T, E> ErrInto<T, E> for Result<T, E>
where
    E: Into<Error>,
{
    fn err_into(self) -> Result<T, Error> {
        self.map_err(Into::into)
    }
}
