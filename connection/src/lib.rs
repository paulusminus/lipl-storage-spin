#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", path = "spin_sqlite.rs")]
mod connection;

#[cfg(target_family = "wasm")]
pub use connection::DbConnection;

#[cfg(not(target_family = "wasm"))]
#[cfg_attr(not(target_family = "wasm"), path = "rusqlite.rs")]
mod connection;

#[cfg(not(target_family = "wasm"))]
pub use connection::DbConnection;
