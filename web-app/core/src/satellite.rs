use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Satellite {
    pub satellite_id: i64,
    pub satellite_name: String,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub body: String,
    pub markdown_body: Option<String>,
    pub is_nsfw: bool,
    pub is_spoiler: bool,
    pub num_posts: i32,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub disable_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}