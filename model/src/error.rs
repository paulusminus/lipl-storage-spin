use axum_core::{body::Body, response::IntoResponse};
use spin_sdk::http::StatusCode;
use std::{num::ParseIntError, str::Utf8Error};

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

    #[cfg(feature = "response")]
    #[error("Decode base64")]
    DecodeBase64(#[from] base64::DecodeError),

    #[error("Utf8: {0}")]
    Utf8(#[from] Utf8Error),

    #[error("Http: {0}")]
    Http(#[from] spin_sdk::http::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Invalid body")]
    Body,

    #[error("Utf 8 encoding")]
    Utf8(#[from] Utf8Error),

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

    #[cfg(feature = "response")]
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

    #[cfg(feature = "response")]
    #[error("Spin SQLite: {0}")]
    SpinSQLite(#[from] spin_sdk::sqlite::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum_core::response::Response {
        match self {
            Error::Authentication(_) => axum_core::response::Response::builder()
                .status(401)
                .header("WWW-Authenticate", "Basic realm=\"Lipl Api\"")
                .body(Body::empty())
                .unwrap(),
            Error::Base58Decode(e) => {
                println!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::Chrono(e) => {
                println!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::Column(e) => {
                println!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::ParseInt(e) => {
                println!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::SpinSQLite(e) => {
                println!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => {
                println!("Unknown error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
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
