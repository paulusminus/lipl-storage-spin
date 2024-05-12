use std::marker::PhantomData;

use rusqlite::{Error, Statement};
use spin_sdk::sqlite::{QueryResult, Row, RowResult, Value};

pub struct DbConnection<E>
where
    E: From<Error>,
{
    inner: rusqlite::Connection,
    phantomdata: PhantomData<E>,
}

impl<E: From<Error>> DbConnection<E> {
    pub(crate) fn try_open_default(migrations: Option<&'static str>) -> Result<Self, E> {
        let connection = rusqlite::Connection::open_in_memory()?;
        if let Some(m) = migrations {
            connection.execute_batch(m)?;
        }
        Ok(Self {
            inner: connection,
            phantomdata: PhantomData::default(),
        })
    }

    pub(crate) fn query<F, S, T>(&self, sql: S, parameters: &[Value], f: F) -> Result<Vec<T>, E>
    where
        F: Fn(Row) -> Result<T, E>,
        S: AsRef<str>,
    {
        let mut statement = rusqlite_statement(&self.inner, sql, parameters)?;
        let columns = statement
            .column_names()
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let mut query_result = QueryResult {
            columns: columns.clone(),
            rows: vec![],
        };
        let mut rows = statement.raw_query();
        while let Some(row) = rows.next()? {
            let mut row_result = RowResult { values: vec![] };
            for column in columns.iter() {
                let field = row.get::<&str, String>(column)?;
                row_result.values.push(Value::Text(field));
            }
            query_result.rows.push(row_result);
        }
        query_result.rows().map(f).collect()
    }

    pub(crate) fn execute<S>(&self, sql: S, parameters: &[Value]) -> Result<i64, E>
    where
        S: AsRef<str>,
    {
        let mut statement = rusqlite_statement(&self.inner, sql, parameters)?;
        let result = statement.raw_execute()?;
        Ok(result.try_into().unwrap())
    }
}

fn rusqlite_statement<'a, S: AsRef<str>, E: From<Error>>(
    connection: &'a rusqlite::Connection,
    sql: S,
    parameters: &[Value],
) -> Result<Statement<'a>, E> {
    let mut statement = connection.prepare(sql.as_ref())?;
    for (i, parameter) in parameters
        .iter()
        .enumerate()
        .map(|(i, value)| (i + 1, value))
    {
        match parameter {
            Value::Blob(blob) => {
                statement.raw_bind_parameter(i, blob)?;
            }
            Value::Integer(integer) => {
                statement.raw_bind_parameter(i, integer)?;
            }
            Value::Null => {
                statement.raw_bind_parameter(i, Option::<String>::None)?;
            }
            Value::Real(float) => {
                statement.raw_bind_parameter(i, float)?;
            }
            Value::Text(s) => {
                statement.raw_bind_parameter(i, s)?;
            }
        };
    }
    Ok(statement)
}
