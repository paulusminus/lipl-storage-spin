# spin-sqlite-connection

## Motivation

I wrote this crate because i wanted to test sqlite database operations in a Fermyon spin application.

This crate uses conditional compilation. The [spin-sdk] crate is used for database operation if target-family = "wasm", else the [rusqlite] crate is used. Then rusqlite connection uses a in-memory database which is used in testing.


## Example

```
    const MIGRATIONS: &str = "CREATE TABLE IF NOT EXISTS user (id TEXT PRIMARY KEY, name TEXT NOT NULL, password TEXT NOT NULL);"
    const INSERT: &str = "INSERT INTO user (id, name, password) VALUES ('LKtQNwbBsQd9aXgMbmptKP', 'paul', 'password');";

    let connection = DbConnection::<Box<dyn std::error::Error>>::try_open_default(Some(MIGRATIONS)).unwrap();
    let count = connection.execute(INSERT, &[]).unwrap();
    assert_eq!(count, 1);
```

[spin-sdk]: https://crates.io/crates/spin-sdk
[rusqlite]: https://crates.io/crates/rusqlite
