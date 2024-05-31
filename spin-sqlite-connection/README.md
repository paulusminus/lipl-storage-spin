# spin-sqlite-connection

## Motivation

I wrote this crate because i wanted to test sqlite database operations in a Fermyon spin application.

This crate uses conditional compilation. The [spin-sdk] crate is used for database operation if target-family = "wasm", else the [rusqlite] crate is used.

[spin-sdk]: https://crates.io/crates/spin-sdk
[rusqlite]: https://crates.io/crates/rusqlite
