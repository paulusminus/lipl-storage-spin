use std::marker::PhantomData;

use rusqlite::{params_from_iter, Error, Params};
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

    pub fn query<S, T, F>(&self, sql: S, parameters: &[Value], f: F) -> Result<Vec<T>, E>
    where
        F: Fn(Row) -> Result<T, E>,
        S: AsRef<str>,
    {
        let mut prepared = self.inner.prepare(sql.as_ref())?;
        let columns = prepared
            .column_names()
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let mut rows = prepared.query(rusqlite_parameters(parameters))?;
        let mut query_result = QueryResult {
            columns,
            rows: vec![],
        };
        while let Some(row) = rows.next()? {
            let mut row_result = RowResult { values: vec![] };
            for column in query_result.columns.iter() {
                let field = row.get::<&str, String>(column)?;
                row_result.values.push(Value::Text(field));
            }
            query_result.rows.push(row_result);
        }
        query_result.rows().map(f).collect()
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