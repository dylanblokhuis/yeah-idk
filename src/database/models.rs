use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostType {
    pub id: String,
    pub singular: String,
    pub plural: String,
    pub path_prefix: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub content: String,
    pub slug: String,
    pub status: String,

    #[serde(rename = "type")]
    pub post_type: String,
}

impl From<PostWithPostType> for Post {
    fn from(post_with_post_type: PostWithPostType) -> Self {
        Post {
            id: post_with_post_type.id,
            title: post_with_post_type.title,
            content: post_with_post_type.content,
            slug: post_with_post_type.slug,
            status: post_with_post_type.status,
            post_type: post_with_post_type.post_type.id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostWithPostType {
    pub id: String,
    pub title: String,
    pub content: String,
    pub slug: String,
    pub status: String,

    #[serde(rename = "type")]
    pub post_type: PostType,
}
