use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct PostType {
    pub id: String,
    pub singular: String,
    pub plural: String,
    pub path_prefix: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub content: String,
    pub slug: String,
    pub status: String,

    #[serde(rename = "type")]
    pub post_type: String,
}
