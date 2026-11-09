use futures_channel::mpsc::channel;
use futures_util::{SinkExt, Stream};
use spin_sdk::sqlite::{QueryResult, RowResult, Value};
use wit_bindgen::spawn_local;

type BoxedError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, BoxedError>;

pub struct SqliteConnection {
    inner: spin_sdk::sqlite::Connection,
}

impl SqliteConnection {
    pub async fn try_open_default(migrations: Option<&str>) -> Result<Self> {
        let connection = spin_sdk::sqlite::Connection::open_default().await?;
        if let Some(statements) = migrations {
            for statement in statements.split('\n').map(|line| line.trim()) {
                connection.execute(statement, []).await?;
            }
        }
        Ok(Self { inner: connection })
    }

    pub async fn query<T>(&self, sql: impl AsRef<str>, parameters: Vec<Value>) -> Result<Vec<T>>
    where
        T: TryFrom<RowResult, Error = BoxedError>,
    {
        let query_result = self.query_result(sql, parameters).await?;
        let rows = query_result.collect().await?;
        rows.into_iter().map(T::try_from).collect()
    }

    async fn query_result(
        &self,
        sql: impl AsRef<str>,
        parameters: Vec<Value>,
    ) -> Result<QueryResult> {
        self.inner
            .execute(sql, parameters)
            .await
            .map_err(Into::into)
    }

    #[allow(dead_code)]
    async fn query_stream<T>(
        &self,
        sql: impl AsRef<str>,
        parameters: Vec<Value>,
    ) -> Result<impl Stream<Item = Result<T>>>
    where
        for<'a> T: TryFrom<RowResult, Error = BoxedError> + 'a,
    {
        let (mut tx, rx) = channel(10);
        let mut query_result = self.query_result(sql, parameters).await?;
        spawn_local(async move {
            while let Some(row) = query_result.next().await {
                tx.send(T::try_from(row)).await.unwrap();
            }
            if let Err(error) = query_result.result().await {
                tx.send(Err(error.into())).await.unwrap();
            }
        });
        Ok(rx)
    }

    pub async fn execute(&self, sql: impl AsRef<str>, parameters: Vec<Value>) -> Result<u64> {
        self.inner.execute(sql, parameters).await?;
        Ok(self.inner.changes().await)
    }

    // async fn changes(&self) -> u64 {
    //     self.inner.changes().await
    // }
}
