use std::path::Path;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::{response::Html, routing::get};
use axum::{Extension, Form, Router};
use axum_flash::{Flash, IncomingFlashes};
use axum_macros::debug_handler;
use serde::Deserialize;
use slugify::slugify;

use crate::database::Db;
use crate::template;

use super::util::TemplateErrors;

pub fn router() -> Router {
    Router::new()
        .route("/", get(admin))
        .route("/posts", get(posts).post(create_post))
        .route("/posts/create", get(create))
}

async fn admin(// Extension(db): Extension<Db>,
    // Extension(runtime): Extension<Runtime>,
) -> Result<Html<String>, StatusCode> {
    let result = template(Path::new("admin.tsx"), "[]".into());

    Ok(result)
}

async fn posts(
    Extension(db): Extension<Db>,
    // Extension(runtime): Extension<Runtime>,
) -> Result<Html<String>, StatusCode> {
    let posts = db.query("SELECT * FROM post FETCH type;").await.unwrap();

    let data = if !posts.is_empty() {
        serde_json::to_string(&posts).unwrap()
    } else {
        "[]".into()
    };
    let result = template(Path::new("admin/posts.tsx"), data);

    Ok(result)
}

#[derive(Debug, Deserialize)]
struct CreatePost {
    title: String,
    content: String,
}

#[debug_handler]
async fn create_post(
    Extension(db): Extension<Db>,
    Form(input): Form<CreatePost>,
    mut flash: Flash,
) -> impl IntoResponse {
    let res = db
        .query(&format!(
            r#"
        CREATE post SET
        title = '{}', 
        content = '{}', 
        slug = '{}',
        created_at = time::now(),
        status = 'draft',
        type = postType:post
        "#,
            input.title,
            input.content,
            slugify!(&input.title)
        ))
        .await;

    if res.is_err() {
        flash.error("Error creating post");
        Redirect::to("/admin/posts/create")
    } else {
        Redirect::to("/admin/posts")
    }
}

#[debug_handler]
async fn create(inc_flash: IncomingFlashes) -> impl IntoResponse {
    template(
        Path::new("admin/posts/create.tsx"),
        serde_json::to_string(&TemplateErrors::from(inc_flash)).unwrap(),
    )
}
