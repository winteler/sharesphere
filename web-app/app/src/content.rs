use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use utils::icons::{FlameIcon, GraphIcon, HourglassIcon, PodiumIcon};

use crate::comment::{Comment, CommentSortType};
use crate::post::{Post, PostSortType};
use crate::ranking::SortType;

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
        false => "whitespace-pre-wrap",
    };

    view! {
        <div
            class=class
            inner_html=body
        />
    }.into_any()
}

/// Component to indicate how to sort posts
#[component]
pub fn PostSortWidget(
    sort_signal: RwSignal<SortType>
) -> impl IntoView {
    view! {
        <div class="join rounded-none">
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Hot) sort_signal datatip="Hot">
                <FlameIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Trending) sort_signal datatip="Trending">
                <GraphIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Best) sort_signal datatip="Best">
                <PodiumIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Recent) sort_signal datatip="Recent">
                <HourglassIcon/>
            </SortWidgetOption>
        </div>
    }.into_any()
}

/// Component to indicate how to sort comments
#[component]
pub fn CommentSortWidget(
    sort_signal: RwSignal<SortType>
) -> impl IntoView {
    view! {
        <div class="join rounded-none">
            <SortWidgetOption sort_type=SortType::Comment(CommentSortType::Best) sort_signal datatip="Best">
                <PodiumIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Comment(CommentSortType::Recent) sort_signal datatip="Recent">
                <HourglassIcon/>
            </SortWidgetOption>
        </div>
    }.into_any()
}

/// Component to show a sorting option
#[component]
pub fn SortWidgetOption(
    sort_type: SortType,
    sort_signal: RwSignal<SortType>,
    datatip: &'static str,
    children: ChildrenFn,
) -> impl IntoView {
    let is_selected = move || sort_signal.read() == sort_type;
    let class = move || {
        let mut class =
            String::from("btn btn-ghost join-item hover:border hover:border-1 hover:border-white ");
        if is_selected() {
            class.push_str("border border-1 border-white ");
        }
        class
    };

    view! {
        <div class="tooltip" data-tip=datatip>
            <button
                class=class
                on:click=move |_| {
                    if sort_signal.get_untracked() != sort_type {
                        sort_signal.set(sort_type);
                    }
                }
            >
                {children()}
            </button>
        </div>
    }.into_any()
}