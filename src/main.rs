use std::net::SocketAddr;
use std::path::Path;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::{response::Html, routing::get};
use axum::{Extension, Router};
use database::Db;

use rquickjs::{Func, Object};
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tsx::compile_app;

mod database;
mod routers;
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
    database::setup_structure(&db).await;

    let app = Router::new()
        .nest("/admin", routers::admin::router())
        .fallback(get(page))
        .layer(Extension(db))
        .layer(axum_flash::layer(axum_flash::Key::generate()).with_cookie_manager())
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

async fn page(
    Extension(db): Extension<Db>,
    request: Request<Body>,
    // Extension(runtime): Extension<Runtime>,
) -> impl IntoResponse {
    let results = db
        .query(&format!(
            "SELECT * FROM post WHERE slug = '{}'",
            request.uri().path().replace('/', "")
        ))
        .await
        .unwrap();

    let val = results.first().unwrap().first();
    if val.is_null() {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(template(
        Path::new("index.tsx"),
        serde_json::to_string(&val).unwrap(),
    ))
}
