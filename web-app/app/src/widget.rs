use leptos::*;

use crate::app::GlobalState;
use crate::comment::CommentSortType;
use crate::constants::{
    SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR,
};
use crate::icons::{
    AuthorIcon, BoldIcon, ClockIcon, FlameIcon, GraphIcon, HourglassIcon, PodiumIcon,
};
use crate::post::PostSortType;
use crate::ranking::SortType;

#[cfg(feature = "ssr")]
mod ssr {
    use std::io::Cursor;
    use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
    use quick_xml::{Reader, Writer};

    use crate::constants::SPOILER_TAG;

    use super::*;

    pub fn style_html_user_content(user_content: &str) -> Result<String, ServerFnError> {
        let mut reader = Reader::from_str(user_content);
        reader.trim_text(true);
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let mut elem = e.clone().into_owned();

                    match elem.name().as_ref() {
                        b"h1" => elem.push_attribute(("class", "text-4xl")),
                        b"h2" => elem.push_attribute(("class", "text-2xl")),
                        b"h3" => elem.push_attribute(("class", "text-xl")),
                        b"a" => elem.push_attribute(("class", "link text-primary")),
                        b"ul" => elem.push_attribute(("class", "list-inside list-disc")),
                        b"ol" => elem.push_attribute(("class", "list-inside list-decimal")),
                        b"code" => elem.push_attribute(("class", "rounded-md bg-black p-1 mx-1")),
                        b"table" => elem.push_attribute(("class", "table")),
                        _ => (),
                    }

                    // writes the event to the writer
                    writer.write_event(Event::Start(elem))?;
                }
                Ok(Event::Empty(e)) => {
                    let mut elem = e.clone().into_owned();

                    match elem.name().as_ref() {
                        b"hr" => elem.push_attribute(("class", "mt-1")),
                        _ => (),
                    }
                    // writes the event to the writer
                    writer.write_event(Event::Start(elem))?;
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap().into_owned();
                    log::info!("Got text in xml: {text}");
                    let spoiler_spitted_text = text.split(SPOILER_TAG);
                    let mut is_current_text_spoiler = None;
                    for text in spoiler_spitted_text {
                        log::info!("Spoiler: {is_current_text_spoiler:?}, {text}");
                        let is_spoiler_text = is_current_text_spoiler.unwrap_or_default();
                        if !text.is_empty() {
                            if is_spoiler_text {
                                let mut elem = BytesStart::new("span");
                                elem.push_attribute(("class", "rounded-md bg-black p-1 mx-1 text-black focus-within:text-white"));
                                writer.write_event(Event::Start(elem))?;

                                writer.write_event(Event::Text(BytesText::new(text.trim())))?;

                                let elem = BytesEnd::new("span");
                                writer.write_event(Event::End(elem))?;
                            } else {
                                writer.write_event(Event::Text(BytesText::new(text)))?;
                            }
                        }
                        is_current_text_spoiler = Some(!is_spoiler_text);
                    }

                    if is_current_text_spoiler.unwrap_or_default() {
                        let elem = BytesEnd::new("span");
                        writer.write_event(Event::End(elem))?;
                    }
                }
                Ok(Event::Eof) => break,
                // we can either move or borrow the event to write, depending on your use-case
                Ok(e) => writer.write_event(e)?,
                Err(e) => {
                    log::error!(
                        "Error while parsing xml at position {}: {:?}",
                        reader.buffer_position(),
                        e
                    );
                    return Err(ServerFnError::from(e));
                }
            }
        }

        let styled_html_output = String::from_utf8(writer.into_inner().into_inner())?;
        Ok(styled_html_output)
    }
}

#[server]
pub async fn get_styled_html_from_markdown(
    markdown_input: String,
) -> Result<String, ServerFnError> {
    let html_from_markdown =
        markdown::to_html_with_options(markdown_input.as_str(), &markdown::Options::gfm())
            .or_else(|e| Err(ServerFnError::new(e)))?;
    log::info!("Markdown as html: {html_from_markdown}");

    // Add styling, will be done by parsing the html which is a bit ugly. Would be better
    // if the styling could be added directly when generating the html from markdown
    let styled_html_output = ssr::style_html_user_content(html_from_markdown.as_str())?;
    Ok(styled_html_output)
}

#[component]
pub fn FormTextEditor(
    name: &'static str,
    placeholder: &'static str,
    #[prop(default = false)] with_publish_button: bool,
) -> impl IntoView {
    let content = create_rw_signal(String::default());

    let render_markdown = create_resource(
        move || content.get(),
        move |markdown_content| get_styled_html_from_markdown(markdown_content),
    );

    view! {
        <div class="group w-full border border-primary rounded-lg bg-base-100">
            <div class="flex max-2xl:flex-col gap-2">
                <div class="w-full 2xl:w-1/2 px-2 py-2 rounded-t-lg">
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
                <div class="spoiler-container">
                    <input type="checkbox" id="spoiler" class="spoiler-input hidden"/>
                    <label for="spoiler" class="spoiler-label cursor-pointer">Reveal Spoiler</label>
                    <div class="spoiler-content">
                        <p class="text-gray-500">"This is the spoiler text. You won't see this until you click!"</p>
                    </div>
                </div>
                <div class="bg-white p-4 rounded shadow">
                    <p class="text-lg font-bold">Click to reveal spoiler:</p>
                    <div class="relative">
                        <input type="checkbox" id="spoiler" class="absolute opacity-0 w-full h-full cursor-pointer top-0 left-0 z-10"/>
                        <label for="spoiler" class="cursor-pointer">Reveal Spoiler</label>
                        <div class="spoiler-content bg-white border border-gray-300 p-4 rounded shadow-md mt-2 opacity-0 transition-opacity duration-300 ease-in-out">
                            <p class="text-gray-500">"This is the spoiler text. You won't see this until you click!"</p>
                        </div>
                    </div>
                </div>
                <Transition>
                    {move || {
                        render_markdown
                            .map(|result| match result {
                                Ok(html) => view! { <div class="break-words w-full 2xl:w-1/2 max-w-prose" inner_html={html}></div> }.into_view(),
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
                        sort_signal.update(|value| *value = sort_type);
                    }
                }
            >
                {children().into_view()}
            </button>
        </div>
    }
}
