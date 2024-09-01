use crate::comment::Comment;
use crate::post::Post;
use leptos::{component, view, IntoView};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Content {
    Post(Post),
    Comment(Comment),
}

/// Displays the body of a content given as input with correct styling for markdown
#[component]
pub fn ContentBody(
    body: String,
    is_markdown: bool,
) -> impl IntoView {
    let class = match is_markdown {
        true => "",
        false => "whitespace-pre",
    };

    view! {
        <div
            class=class
            inner_html=body
        />
    }
}