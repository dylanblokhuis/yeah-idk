use surrealdb::{sql::Value, Datastore, Error, Session};
use tokio::sync::{mpsc, oneshot};

pub mod models;

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

    pub async fn query_first<T: for<'de> serde::Deserialize<'de>>(
        &self,
        statement: &str,
    ) -> Result<T, Error> {
        let (tx, rx) = oneshot::channel();

        self.query_sender
            .send((tx, statement.to_string()))
            .await
            .unwrap();

        let responses = rx.await.unwrap()?;
        let response = responses.first().expect("Query returned nothing");
        let result = response.result.as_ref().unwrap();

        let val = serde_json::to_value(result).unwrap();

        Ok(
            serde_json::from_value::<T>(val.clone()).unwrap_or_else(|_| {
                panic!(
                    "Failed to convert to JSON, statement: '{}', results: {:?}",
                    statement, val
                )
            }),
        )
    }
}

pub async fn setup_structure(db: &Db) {
    // just for debugging
    // dbg!(
    //     "{:?}",
    //     db.query(
    //         r#"
    //     SELECT * FROM postType FETCH post
    // "#
    //     )
    //     .await
    //     .unwrap()
    // );

    let res = db.query("SELECT * FROM postType").await.unwrap();
    if !res.is_empty() && res.first().unwrap().is_truthy() {
        return;
    }

    db.query("CREATE postType:page SET singular = 'Page', plural = 'Pages'")
        .await
        .unwrap();

    db.query(
        "CREATE postType:post SET singular = 'Post', plural = 'Posts', path_prefix = '/posts/'",
    )
    .await
    .unwrap();
}
