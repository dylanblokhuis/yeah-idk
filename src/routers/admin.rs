use std::path::Path;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::{response::Html, routing::get};
use axum::{Extension, Form, Router};
use axum_flash::{Flash, IncomingFlashes};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use slugify::slugify;

use crate::database::models::{Post, PostType};
use crate::database::Db;
use crate::template;

use super::util::{TemplateError, TemplateErrors};

pub fn router() -> Router {
    Router::new()
        .route("/", get(admin))
        .route("/posts", get(posts).post(create_post))
        .route("/posts/create", get(create))
}

async fn admin(
    Extension(db): Extension<Db>,
    // Extension(runtime): Extension<Runtime>,
) -> Result<Html<String>, StatusCode> {
    let post_types = db
        .query_first::<Vec<PostType>>("SELECT * FROM postType")
        .await
        .unwrap();

    let result = template(
        Path::new("admin.tsx"),
        serde_json::to_string(&post_types).unwrap(),
    );

    Ok(result)
}

#[derive(Deserialize)]
struct PostsQueryParams {
    #[serde(rename = "type")]
    post_type: Option<String>,
}

#[derive(Serialize)]
struct Posts {
    post_type: PostType,
    posts: Vec<Post>,
}

async fn posts(
    Extension(db): Extension<Db>,
    Query(query): Query<PostsQueryParams>,
) -> impl IntoResponse {
    let post_type = match query.post_type {
        Some(post_type) => post_type,
        None => "postType:post".into(),
    };

    let maybe_post_type = db
        .query_first::<Vec<PostType>>(&format!("SELECT * FROM {}", post_type))
        .await
        .unwrap();

    let post_type = match maybe_post_type.first() {
        Some(post_type) => post_type,
        None => return Err(StatusCode::NOT_FOUND),
    };

    let posts = db
        .query_first::<Vec<Post>>(&format!("SELECT * FROM post WHERE type = {}", post_type.id))
        .await
        .unwrap();

    let result = Posts {
        post_type: post_type.clone(),
        posts,
    };

    Ok(template(
        Path::new("admin/posts.tsx"),
        serde_json::to_string(&result).unwrap(),
    ))
}

#[derive(Debug, Deserialize)]
struct CreatePost {
    title: String,
    content: String,
    post_type: String,
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
        status = 'published',
        type = {}
        "#,
            input.title,
            input.content,
            slugify!(&input.title),
            input.post_type
        ))
        .await;

    if res.is_err() {
        flash.error("Error creating post");
        Redirect::to("/admin/posts/create")
    } else {
        Redirect::to("/admin/posts")
    }
}

#[derive(Serialize)]
struct CreateResponse {
    post_type: PostType,
    errors: Vec<TemplateError>,
}
#[debug_handler]
async fn create(
    inc_flash: IncomingFlashes,
    Query(query): Query<PostsQueryParams>,
    Extension(db): Extension<Db>,
) -> impl IntoResponse {
    let post_type = match query.post_type {
        Some(post_type) => post_type,
        None => "postType:post".into(),
    };

    let maybe_post_type = db
        .query_first::<Vec<PostType>>(&format!("SELECT * FROM {}", post_type))
        .await
        .unwrap();

    let post_type = match maybe_post_type.first() {
        Some(post_type) => post_type,
        None => return Err(StatusCode::NOT_FOUND),
    };

    Ok(template(
        Path::new("admin/posts/create.tsx"),
        serde_json::to_string(&CreateResponse {
            post_type: post_type.clone(),
            errors: TemplateErrors::from(inc_flash).errors,
        })
        .unwrap(),
    ))
}
