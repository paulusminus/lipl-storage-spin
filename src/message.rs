use std::fmt::Display;

use crate::{now, request_id};

pub fn db_connection_established() {
    println!(
        "{}: connection with default sqlite db established after {} milliseconds",
        request_id(),
        now().elapsed().as_millis(),
    );
}

pub fn request_received(path: impl Display) {
    println!(
        "{}: received request {} after {} milliseconds",
        request_id(),
        path,
        now().elapsed().as_millis()
    );
}

pub fn update_playlist_failure(id: impl Display) {
    eprintln!(
        "{}: Failed to update title and modified for playlist with id {id}",
        request_id()
    );
}

pub fn insert_playlist_failure(error: impl Display) {
    eprintln!(
        "{}: Error while inserting new playlist: {error}",
        request_id()
    );
}

pub fn rollback_failure(error: impl Display) {
    eprintln!("{}: Cannot rollback {}", request_id(), error);
}
