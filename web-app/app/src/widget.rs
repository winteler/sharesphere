use leptos::*;

use crate::app::GlobalState;
use crate::comment::{CommentSortType};
use crate::constants::{SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR};
use crate::icons::{AuthorIcon, BoldIcon, ClockIcon, FlameIcon, GraphIcon, HourglassIcon, PodiumIcon};
use crate::post::PostSortType;
use crate::ranking::SortType;

#[server]
pub async fn get_html_from_markdown(
    markdown_content: String,
) -> Result<String, ServerFnError> {
    use femark::{process_markdown_to_html};

    match process_markdown_to_html(markdown_content.as_str()) {
        Ok(render) => {
            log::info!("Markdown render: {}", render.content);
            Ok(render.content)
        }
        Err(e) => {
            log::error!("Failed to render markdown with error: {e}");
            Err(ServerFnError::new("Failed to render markdown to html"))
        }
    }
}

#[component]
pub fn FormTextEditor(
    name: &'static str,
    placeholder: &'static str,
    #[prop(default = false)]
    with_publish_button: bool,
) -> impl IntoView {
    use markdown_to_html_parser::parse_markdown;
    let content = create_rw_signal(String::default());

    let render_markdown = create_resource(
        move || content.get(),
        move |markdown_content| get_html_from_markdown(markdown_content)
    );

    let markdown_render = move || {
        let content = content();
        let markdown_content = parse_markdown(content.as_str());
        log::info!("Markdown as html: {markdown_content}");
        markdown_content
    };

    view! {
        <div class="group w-full border border-primary rounded-lg bg-base-100">
            <div class="flex gap-2">
                <div class="px-2 py-2 rounded-t-lg">
                    <label for="comment" class="sr-only">
                        "Your comment"
                    </label>
                    <textarea
                        id="comment"
                        name=name
                        placeholder=placeholder
                        class="w-full px-0 bg-base-100 outline-none border-none"
                        on:input=move |ev| {
                            content.update(|content: &mut String| *content = event_target_value(&ev));
                        }
                    ></textarea>

                </div>
                <div inner_html=markdown_render></div>
                <Transition>

                    {move || {
                        render_markdown
                            .map(|result| match result {
                                Ok(html) => view! { <div inner_html=html></div> }.into_view(),
                                Err(_) => view! { <div>"Failed to parse markdown"</div> }.into_view(),
                            })
                    }}

                </Transition>
            </div>
            <div class="flex justify-between px-2">
                <div class="flex">
                    <button type="button" class="btn btn-ghost">
                        <BoldIcon/>
                    </button>
                </div>
                <button class="btn btn-active btn-secondary" class:hidden=move || !with_publish_button disabled=move || content().is_empty()>
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
                    seconds if seconds < SECONDS_IN_MINUTE => format!("{} {}", seconds, if seconds == 1 { "second" } else { "seconds" }),
                    seconds if seconds < SECONDS_IN_HOUR => {
                        let minutes = seconds / SECONDS_IN_MINUTE;
                        format!("{} {}", minutes, if minutes == 1 { "minute" } else { "minutes" })
                    }
                    seconds if seconds < SECONDS_IN_DAY => {
                        let hours = seconds / SECONDS_IN_HOUR;
                        format!("{} {}", hours, if hours == 1 { "hour" } else { "hours" })
                    }
                    seconds if seconds < SECONDS_IN_MONTH => {
                        let days = seconds / SECONDS_IN_DAY;
                        format!("{} {}", days, if days == 1 { "day" } else { "days" })
                    }
                    seconds if seconds < SECONDS_IN_YEAR => {
                        let months = seconds / SECONDS_IN_MONTH;
                        format!("{} {}", months, if months == 1 { "month" } else { "months" })
                    }
                    _ => {
                        let years = seconds / SECONDS_IN_YEAR;
                        format!("{} {}", years, if years == 1 { "year" } else { "years" })
                    }
                }
            }

        </div>
    }
}

/// Component to indicate how to sort posts
#[component]
pub fn PostSortWidget() -> impl IntoView {
    view! {
        <div class="join rounded-none">
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Hot) datatip="Hot">
                <FlameIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Trending) datatip="Trending">
                <GraphIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Best) datatip="Best">
                <PodiumIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Post(PostSortType::Recent) datatip="Recent">
                <HourglassIcon/>
            </SortWidgetOption>
        </div>
    }
}

/// Component to indicate how to sort comments
#[component]
pub fn CommentSortWidget() -> impl IntoView {
    view! {
        <div class="join rounded-none">
            <SortWidgetOption sort_type=SortType::Comment(CommentSortType::Best) datatip="Best">
                <PodiumIcon/>
            </SortWidgetOption>
            <SortWidgetOption sort_type=SortType::Comment(CommentSortType::Recent) datatip="Recent">
                <HourglassIcon/>
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
    let class = move || {
        let mut class = String::from("btn btn-ghost join-item hover:border hover:border-1 hover:border-white ");
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
                        sort_signal.update(|value| *value = sort_type);
                    }
                }
            >
                {children().into_view()}
            </button>
        </div>
    }
}