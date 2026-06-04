use spin_sdk::sqlite::{Error, QueryResult, RowResult, Value};
use std::marker::PhantomData;

pub struct SqliteConnection<E>
where
    E: From<Error>,
{
    inner: spin_sdk::sqlite::Connection,
    phantomdata: PhantomData<E>,
}

impl<E: From<Error>> SqliteConnection<E> {
    pub async fn try_open_default(migrations: Option<&str>) -> Result<Self, E> {
        let connection = spin_sdk::sqlite::Connection::open_default().await?;
        if let Some(statements) = migrations {
            for statement in statements.split('\n').map(|line| line.trim()) {
                connection.execute(statement, []).await?;
            }
        }
        Ok(Self {
            inner: connection,
            phantomdata: PhantomData,
        })
    }

    pub async fn query<T>(&self, sql: impl AsRef<str>, parameters: Vec<Value>) -> Result<Vec<T>, E>
    where
        T: TryFrom<RowResult, Error = E>,
    {
        let query_result = self.query_result(sql, parameters).await?;
        let rows = query_result.collect().await?;
        rows.into_iter().map(T::try_from).collect()
    }

    async fn query_result<S>(&self, sql: S, parameters: Vec<Value>) -> Result<QueryResult, E>
    where
        S: AsRef<str>,
    {
        self.inner
            .execute(sql.as_ref(), parameters)
            .await
            .map_err(E::from)
    }

    pub async fn execute<S>(&self, sql: S, parameters: Vec<Value>) -> Result<u64, E>
    where
        S: AsRef<str>,
    {
        self.inner.execute(sql.as_ref(), parameters).await?;
        Ok(self.inner.changes().await)
    }
}
