use std::{marker::PhantomData, str::from_utf8};

use rusqlite::{params_from_iter, types::ValueRef, Error, Params};
use spin_sdk::sqlite::{QueryResult, Row, RowResult, Value};

pub struct DbConnection<E>
where
    E: From<Error>,
{
    inner: rusqlite::Connection,
    phantomdata: PhantomData<E>,
}

impl<E: From<Error>> DbConnection<E> {
    pub fn try_open_default(migrations: Option<&'static str>) -> Result<Self, E> {
        let connection = rusqlite::Connection::open_in_memory()?;
        if let Some(m) = migrations {
            connection.execute_batch(m)?;
        }
        Ok(Self {
            inner: connection,
            phantomdata: PhantomData,
        })
    }

    pub fn query<T>(&self, sql: impl AsRef<str>, parameters: &[Value]) -> Result<Vec<T>, E>
    where
        T: for<'a> TryFrom<Row<'a>, Error = E>,
    {
        self.query_rows(sql, parameters)
            .and_then(|query_result| query_result.rows().map(T::try_from).collect())
    }

    fn query_rows<S>(&self, sql: S, parameters: &[Value]) -> Result<QueryResult, E>
    where
        S: AsRef<str>,
    {
        let mut prepared = self.inner.prepare(sql.as_ref())?;
        let columns = prepared
            .column_names()
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let rows = prepared
            .query_map(rusqlite_parameters(parameters), |row| {
                (0..columns.len())
                    .map(|i| row.get_ref(i).and_then(spin_sqlite_value))
                    .collect::<Result<Vec<_>, _>>()
                    .map(|values| RowResult { values })
            })
            .and_then(|mapped_rows| mapped_rows.collect::<Result<Vec<_>, Error>>())?;
        Ok(QueryResult { columns, rows })
    }

    pub fn execute<S>(&self, sql: S, parameters: &[Value]) -> Result<i64, E>
    where
        S: AsRef<str>,
    {
        let count = self
            .inner
            .execute(sql.as_ref(), rusqlite_parameters(parameters))?;
        Ok(count.try_into().unwrap())
    }
}

fn spin_sqlite_value(value: ValueRef) -> Result<spin_sdk::sqlite::Value, rusqlite::Error> {
    match value {
        ValueRef::Blob(blob) => Ok(Value::Blob(blob.to_vec())),
        ValueRef::Integer(integer) => Ok(Value::Integer(integer)),
        ValueRef::Real(real) => Ok(Value::Real(real)),
        ValueRef::Null => Ok(Value::Null),
        ValueRef::Text(text) => from_utf8(text)
            .map(String::from)
            .map(Value::Text)
            .map_err(Error::Utf8Error),
    }
}

fn rusqlite_parameter(parameter: &Value) -> rusqlite::types::Value {
    match parameter {
        Value::Blob(blob) => rusqlite::types::Value::Blob(blob.clone()),
        Value::Integer(integer) => rusqlite::types::Value::Integer(*integer),
        Value::Null => rusqlite::types::Value::Null,
        Value::Real(real) => rusqlite::types::Value::Real(*real),
        Value::Text(text) => rusqlite::types::Value::Text(text.clone()),
    }
}

fn rusqlite_parameters(parameters: &[Value]) -> impl Params {
    params_from_iter(
        parameters
            .iter()
            .map(rusqlite_parameter)
            .collect::<Vec<_>>(),
    )
}
