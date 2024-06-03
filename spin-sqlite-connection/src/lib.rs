#![doc = include_str!("../README.md")]

#[cfg(wasm)]
#[cfg_attr(wasm, path = "spin_sqlite.rs")]
mod connection;

#[cfg(not_wasm)]
#[cfg_attr(not_wasm, path = "rusqlite.rs")]
mod connection;

pub use connection::SqliteConnection;
