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
    pub fn try_open_default(migrations: Option<&'static str>) -> Result<Self, E> {
        let connection = spin_sdk::sqlite::Connection::open_default()?;
        if migrations.is_some() {
            unimplemented!();
        }
        Ok(Self {
            inner: connection,
            phantomdata: PhantomData,
        })
    }

    pub fn query<F, S, T>(&self, sql: S, parameters: &[Value]) -> Result<Vec<T>, E>
    where
        F: Fn(Row) -> Result<T, E>,
        S: AsRef<str>,
    {
        self.inner
            .execute(sql.as_ref(), parameters)
            .map_err(E::from)
            .and_then(|result| result.rows().map(f).collect())
    }

    pub fn execute<S>(&self, sql: S, parameters: &[Value]) -> Result<i64, E>
    where
        S: AsRef<str>,
    {
        self.inner.execute(sql.as_ref(), parameters)?;
        let changes = self.inner.execute("SELECT changes()", &[])?;
        let row = changes
            .rows
            .first()
            .cloned()
            .ok_or(Error::Io("changes() has no rows".to_owned()))?;
        let count = row
            .get::<i64>(0)
            .ok_or(Error::Io("Column changes() missing".to_owned()))?;
        Ok(count)
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn migration() {
        let connection = super::DbConnection::try_open_default(None).unwrap();
    }

    #[test]
    fn migration_whatever() {
        let migration = super::DbConnection::try_open_default(Some(include_str!("../migrations.sql"))).unwrap();
    }
}
