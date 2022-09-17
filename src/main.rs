use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::response::Result;
use axum::{response::Html, routing::get};
use axum::{Extension, Router};
use database::Db;
use rusty_jsc::JSContext;

use tsx::compile_app;

mod database;
mod tsx;

#[tokio::main]
async fn main() {
    let db = Db::new("test".into(), "test".into(), "file://temp.db".into()).await;

    let app = Router::new().route("/", get(response)).layer(Extension(db));

    // run it asdasd
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn response(Extension(db): Extension<Db>) -> Result<Html<String>, StatusCode> {
    let res = db.query("SELECT * FROM account;").await;

    println!("{:?}", res);

    let js = compile_app();

    //println!("js: {}", js);
    std::fs::write("./out.js", &js).unwrap();
    let mut context = JSContext::default();
    match context.evaluate_script(&js, 1) {
        Some(value) => Ok(Html(value.to_string(&context))),
        None => {
            println!(
                "Uncaught: {}",
                context.get_exception().unwrap().to_string(&context)
            );

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
