use serde::{Deserialize, Serialize};
use sharesphere_utils::embed::Link;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Post {
    pub post_id: i64,
    pub title: String,
    pub body: String,
    pub markdown_body: Option<String>,
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub link: Link,
    pub is_nsfw: bool,
    pub is_spoiler: bool,
    pub category_id: Option<i64>,
    pub is_edited: bool,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub satellite_id: Option<i64>,
    pub creator_id: i64,
    pub creator_name: String,
    pub is_creator_moderator: bool,
    pub moderator_message: Option<String>,
    pub infringed_rule_id: Option<i64>,
    pub infringed_rule_title: Option<String>,
    pub moderator_id: Option<i64>,
    pub moderator_name: Option<String>,
    pub num_comments: i32,
    pub is_pinned: bool,
    pub score: i32,
    pub score_minus: i32,
    pub recommended_score: f32,
    pub trending_score: f32,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub scoring_timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl Post {
    pub fn is_active(&self) -> bool {
        self.delete_timestamp.is_none() && self.moderator_id.is_none()
    }
}