use leptos::*;
use serde::{Deserialize, Serialize};
use crate::app::GlobalState;
use crate::comment::CommentSortType;

use crate::constants::{SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR};
use crate::icons::{AuthorIcon, BoldIcon, BonfireIcon, ClockIcon, FlameIcon, GraphIcon, HotIcon, HourglassIcon, MedalIcon, MedalIcon2, PodiumIcon, PodiumIcon2, SimpleFlameIcon, SimpleFlameIcon2, TimewatchIcon};
use crate::post::PostSortType;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum SortType {
    Post(PostSortType),
    Comment(CommentSortType),
}

#[component]
pub fn FormTextEditor(
    name: &'static str,
    placeholder: &'static str,
    #[prop(default = false)]
    with_publish_button: bool,
) -> impl IntoView {
    let is_empty = create_rw_signal(true);

    view! {
        <div class="group w-full my-2 border border-primary rounded-lg bg-base-100">
            <div class="px-2 py-2 rounded-t-lg">
                <label for="comment" class="sr-only">"Your comment"</label>
                <textarea
                    id="comment"
                    name=name
                    placeholder=placeholder
                    class="w-full px-0 bg-base-100 outline-none border-none"
                    on:input=move |ev| {
                        is_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                    }
                />
            </div>

            <div
                class="flex justify-between px-2 pb-2"
            >
                <div class="flex">
                    <button
                        type="button"
                        class="btn btn-ghost"
                    >
                        <BoldIcon/>
                    </button>
                </div>
                <button
                    class="btn btn-active btn-secondary"
                    class:hidden=move || !with_publish_button
                    disabled=is_empty
                >
                    "Publish"
                </button>
            </div>
        </div>
    }
}

/// Component to display the author of a post or comment
#[component]
pub fn AuthorWidget<'a>(author: &'a String) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center text-sm">
            <AuthorIcon/>
            {author.clone()}
        </div>
    }
}

/// Component to display the creation time of a post
#[component]
pub fn TimeSinceWidget<'a>(timestamp: &'a chrono::DateTime<chrono::Utc>) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center text-sm">
            <ClockIcon/>
            {
                let elapsed_time = chrono::Utc::now().signed_duration_since(timestamp);
                let seconds = elapsed_time.num_seconds();

                match seconds {
                    seconds if seconds < SECONDS_IN_MINUTE => {
                        format!("{} {}", seconds, if seconds == 1 { "second" } else { "seconds" })
                    },
                    seconds if seconds < SECONDS_IN_HOUR => {
                        let minutes = seconds/SECONDS_IN_MINUTE;
                        format!("{} {}", minutes, if minutes == 1 { "minute" } else { "minutes" })
                    },
                    seconds if seconds < SECONDS_IN_DAY => {
                        let hours = seconds/SECONDS_IN_HOUR;
                        format!("{} {}", hours, if hours == 1 { "hour" } else { "hours" })
                    },
                    seconds if seconds < SECONDS_IN_MONTH => {
                        let days = seconds/SECONDS_IN_DAY;
                        format!("{} {}", days, if days == 1 { "day" } else { "days" })
                    },
                    seconds if seconds < SECONDS_IN_YEAR => {
                        let months = seconds/SECONDS_IN_MONTH;
                        format!("{} {}", months, if months == 1 { "month" } else { "months" })
                    },
                    _ => {
                        let years = seconds/SECONDS_IN_YEAR;
                        format!("{} {}", years, if years == 1 { "year" } else { "years" })
                    },
                }
            }
        </div>
    }
}

/// Component to indicate how to sort contents
#[component]
pub fn PostSortWidget() -> impl IntoView {
    view! {
        <div class="join">
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Hot) datatip="Hot">
                <HotIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Hot) datatip="Hot">
                <SimpleFlameIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Hot) datatip="Hot">
                <FlameIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Hot) datatip="Hot">
                <BonfireIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Hot) datatip="Hot">
                <SimpleFlameIcon2/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Best) datatip="Best">
                <PodiumIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Best) datatip="Best">
                <PodiumIcon2/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Best) datatip="Best">
                <MedalIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Best) datatip="Best">
                <MedalIcon2/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Trending) datatip="Trending">
                <GraphIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Recent) datatip="Recent">
                <HourglassIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Recent) datatip="Recent">
                <TimewatchIcon/>
            </SortWidgetOption>
        </div>
    }
}

/// Component to show a sorting option
#[component]
pub fn SortWidgetOption(
    sort_type: SortType,
    datatip: &'static str,
    children: ChildrenFn,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sort_signal = match sort_type {
        SortType::Post(_) => state.post_sort_type,
        SortType::Comment(_) => state.comment_sort_type,
    };
    let is_selected = move || sort_type == sort_signal.get();
    let class = move || match is_selected() {
        true => "btn btn-ghost border border-1 border-primary hover:bg-white join-item",
        false => "btn btn-ghost hover:bg-white join-item"
    };

    view! {
        <div class="tooltip" data-tip=datatip>
            <button
                class=class
                on:click=move |_| {
                    if sort_signal.get_untracked() != sort_type {
                        sort_signal.update(|value| *value = sort_type);
                    }
                }
            >
                { children().into_view() }
            </button>
        </div>
    }
}