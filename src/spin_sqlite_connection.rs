use spin_sdk::sqlite::{Error, Row, Value};
use std::marker::PhantomData;

pub struct DbConnection<E>
where
    E: From<Error>,
{
    inner: spin_sdk::sqlite::Connection,
    phantomdata: PhantomData<E>,
}

impl<E: From<Error>> DbConnection<E> {
    pub(crate) fn try_open_default(migrations: Option<&'static str>) -> Result<Self, E> {
        let connection = spin_sdk::sqlite::Connection::open_default()?;
        if let Some(m) = migrations {
            connection.execute(m, &[])?;
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
        self.inner
            .execute(sql.as_ref(), parameters)
            .map_err(E::from)
            .and_then(|result| result.rows().map(f).collect())
    }

    pub(crate) fn execute<S>(&self, sql: S, parameters: &[Value]) -> Result<i64, E>
    where
        S: AsRef<str>,
    {
        self.inner.execute(sql.as_ref(), parameters)?;
        let changes = self.inner.execute("SELECT changes()", &[])?;
        match changes.rows.first().cloned() {
            Some(row) => {
                // using i64 is crucial !!!
                let count = row
                    .get::<i64>(0)
                    .ok_or(Error::Io("Column changes() missing".to_owned()))?;
                Ok(count)
            }
            None => Ok(0),
        }
    }
}
