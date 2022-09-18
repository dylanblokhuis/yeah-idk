use std::net::SocketAddr;
use std::path::Path;

use axum::http::StatusCode;
use axum::response::Result;
use axum::{response::Html, routing::get};
use axum::{Extension, Router};
use database::Db;

use rquickjs::{Func, Object};
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tsx::compile_app;

mod database;
mod tsx;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "experimental-cms=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Db::new("test".into(), "test".into(), "file://temp.db".into()).await;
    let runtime = rquickjs::Runtime::new().unwrap();

    let app = Router::new()
        .route("/", get(home))
        .route("/admin", get(admin))
        .route("/admin/posts", get(admin_posts))
        .layer(Extension(db))
        .layer(Extension(runtime))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn print(msg: String) {
    println!("{}", msg);
}

fn template(path: &Path, data: String) -> Html<String> {
    let js = compile_app(path, data);

    std::fs::write("out.js", &js).unwrap();

    let runtime = rquickjs::Runtime::new().unwrap();
    let ctx = rquickjs::Context::full(&runtime).unwrap();
    let result = ctx.with(|ctx| {
        let global = ctx.globals();
        let obj = Object::new(ctx).unwrap();
        obj.set("log", Func::new("print", print)).unwrap();
        global.set("console", obj).unwrap();

        ctx.eval::<String, String>(js).unwrap()
    });

    Html(result)
}

async fn home(// Extension(db): Extension<Db>,
    // Extension(runtime): Extension<Runtime>,
) -> Result<Html<String>, StatusCode> {
    let result = template(Path::new("index.tsx"), "[]".into());

    Ok(result)
}

async fn admin(// Extension(db): Extension<Db>,
    // Extension(runtime): Extension<Runtime>,
) -> Result<Html<String>, StatusCode> {
    let result = template(Path::new("admin.tsx"), "[]".into());

    Ok(result)
}

async fn admin_posts(
    Extension(db): Extension<Db>,
    // Extension(runtime): Extension<Runtime>,
) -> Result<Html<String>, StatusCode> {
    db.query("CREATE post SET name = 'Hello World!', content = 'This is the content';")
        .await
        .unwrap();

    let posts = db.query("SELECT * FROM post").await.unwrap();

    // for post in posts {
    //     println!("{}", post.as_string());
    //     // post.ser
    // }

    let data = serde_json::to_string(&posts).unwrap();

    let result = template(Path::new("admin/posts.tsx"), data);

    Ok(result)
}
