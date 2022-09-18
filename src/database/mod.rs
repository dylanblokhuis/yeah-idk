use surrealdb::{sql::Value, Datastore, Error, Session};
use tokio::sync::{mpsc, oneshot};

type QueryResult = (
    oneshot::Sender<Result<Vec<surrealdb::Response>, Error>>,
    String,
);

#[derive(Clone)]
pub struct Db {
    pub query_sender: mpsc::Sender<QueryResult>,
}

impl Db {
    pub async fn new(namespace: String, database: String, datastore: String) -> Self {
        let (stmt_tx, mut stmt_rx) = mpsc::channel::<QueryResult>(32);

        tokio::spawn(async move {
            let datastore = Datastore::new(&datastore).await.unwrap();
            let session = Session::for_db(namespace.to_string(), database.to_string());

            while let Some(statement) = stmt_rx.recv().await {
                let (tx, query) = statement;
                let responses = datastore.execute(&query, &session, None, false).await;
                tx.send(responses).unwrap();
            }
        });

        Self {
            query_sender: stmt_tx,
        }
    }

    pub async fn query(&self, statement: &str) -> Result<Vec<Value>, Error> {
        let (tx, rx) = oneshot::channel();

        self.query_sender
            .send((tx, statement.to_string()))
            .await
            .unwrap();

        let responses = rx.await.unwrap()?;
        let mut results = Vec::new();

        for response in responses {
            if let Ok(value) = response.result {
                results.push(value);
            }
        }

        Ok(results)
    }

    // pub async fn query<'a>(&self, statement: &'a str) -> Result<Vec<Value>, Error> {
    //     Ok(results)
    // }
}
