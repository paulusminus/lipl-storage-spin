use std::{fmt::Display, sync::OnceLock, time::Instant};

use model::Uuid;

pub fn request_id() -> &'static Uuid {
    static REQUEST_ID: OnceLock<Uuid> = OnceLock::new();
    REQUEST_ID.get_or_init(Uuid::default)
}

pub fn now() -> &'static Instant {
    static NOW: OnceLock<Instant> = OnceLock::new();
    NOW.get_or_init(Instant::now)
}

pub fn db_connection_established() {
    println!(
        "{}: connection with default sqlite db established after {} microseconds",
        request_id(),
        now().elapsed().as_micros(),
    );
}

pub fn user_authenticated(user: impl Display) {
    println!(
        "{}: User {} after {} milliseconds",
        request_id(),
        user,
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

pub fn dump_header(name: &str, value: &str) {
    println!("{}: {} = {}", request_id(), name, value);
}
