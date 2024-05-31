use std::fmt::Display;

use crate::{now, request_id};

pub fn db_connection_established() {
    println!(
        "{}: connection with default sqlite db established after {} milliseconds",
        request_id(),
        now().elapsed().as_millis(),
    );
}

pub fn request_received(path: impl Display, method: impl Display) {
    println!(
        "{}: received {} {} request after {} milliseconds",
        request_id(),
        method.to_string().to_lowercase(),
        path,
        now().elapsed().as_millis()
    );
}

pub fn rollback_failure(error: impl Display) {
    eprintln!("{}: Cannot rollback {}", request_id(), error);
}
