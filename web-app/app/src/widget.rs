use leptos::*;

use crate::constants::{SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR};
use crate::icons::{AuthorIcon, BoldIcon, ClockIcon, FlameIcon, HotIcon};

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
pub fn SortWidget() -> impl IntoView {
    view! {
        <ul class="menu menu-horizontal rounded-box">
            <li>
                <a class="tooltip" data-tip="Home">
                    <HotIcon/>
                </a>
            </li>
            <li>
                <a class="tooltip" data-tip="Details">
                    <FlameIcon/>
                </a>
            </li>
            <li>
                <a class="tooltip" data-tip="Stats">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" /></svg>
                </a>
            </li>
        </ul>
    }
}