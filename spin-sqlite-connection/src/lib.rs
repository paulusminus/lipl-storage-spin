#![doc = include_str!("../README.md")]

#[cfg(wasm)]
mod spin_sqlite;

#[cfg(not(wasm))]
mod turso_sqlite;

#[cfg(wasm)]
pub use spin_sqlite::SqliteConnection;

#[cfg(not(wasm))]
pub use turso_sqlite::SqliteConnection;
