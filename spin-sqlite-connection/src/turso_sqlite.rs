use std::{marker::PhantomData, str::from_utf8};

use spin_sdk::sqlite::{QueryResult, Value};
use turso::{Error, IntoParams};

pub struct SqliteConnection<E>
where
    E: From<Error>,
{
    inner: turso::Connection,
    phantomdata: PhantomData<E>,
}

impl<E: From<Error>> SqliteConnection<E> {
    pub async fn try_open_default(migrations: Option<&str>) -> Result<Self, E> {
        let database = turso::Builder::new_local(":memory:").build().await?;
        let connection = database.connect()?;
        if let Some(migrations) = migrations {
            connection.execute_batch(migrations).await?;
        }
        Ok(Self {
            inner: connection,
            phantomdata: PhantomData,
        })
    }

    async fn query_result<S>(&self, sql: S, parameters: Vec<Value>) -> Result<QueryResult, E>
    where
        S: AsRef<str>,
    {
        let rows = self
            .inner
            .query(sql.as_ref(), turso_parameters(&parameters))
            .await?;
        while let Some(row) = rows.next().await? {}
    }
}

fn spin_sqlite_value(
    value: turso::Value,
    _column: usize,
) -> Result<spin_sdk::sqlite::Value, turso::Error> {
    match value {
        turso::Value::Blob(blob) => Ok(Value::Blob(blob.to_vec())),
        turso::Value::Integer(integer) => Ok(Value::Integer(integer)),
        turso::Value::Real(real) => Ok(Value::Real(real)),
        turso::Value::Null => Ok(Value::Null),
        turso::Value::Text(text) => Ok(Value::Text(text)),
    }
}

fn turso_parameter(parameter: &Value) -> turso::Value {
    match parameter {
        Value::Blob(blob) => turso::Value::Blob(blob.clone()),
        Value::Integer(integer) => turso::Value::Integer(*integer),
        Value::Null => turso::Value::Null,
        Value::Real(real) => turso::Value::Real(*real),
        Value::Text(text) => turso::Value::Text(text.clone()),
    }
}

fn turso_parameters(parameters: &[Value]) -> impl IntoParams + use<> {
    turso::params_from_iter(parameters.iter().map(turso_parameter).collect::<Vec<_>>())
}
