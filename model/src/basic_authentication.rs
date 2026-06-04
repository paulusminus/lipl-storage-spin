use spin_sdk::{
    http::{Response, StatusCode},
    wasip3::http_compat::http_into_wasi_response,
};

pub fn unauthenticated() -> spin_sdk::wasip3::http::types::Response {
    let response = Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Basic realm=\"Lipl Api\"")
        .body(String::new())
        .unwrap();
    http_into_wasi_response(response).unwrap()
}
