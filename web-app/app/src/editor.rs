use leptos::html;
use leptos::prelude::*;
use leptos_use::{on_click_outside, signal_debounced};

use crate::constants::SPOILER_TAG;
use crate::errors::AppError;
use crate::icons::*;
use crate::unpack::TransitionUnpack;

#[derive(Clone, Copy, Debug)]
pub enum FormatType {
    Bold,
    Italic,
    Strikethrough,
    Header1,
    Header2,
    List,
    NumberedList,
    CodeBlock,
    Spoiler,
    BlockQuote,
    Link,
    Image,
}

#[derive(Clone, Copy, Debug)]
pub struct TextareaData {
    pub content: Signal<String>,
    pub set_content: WriteSignal<String>,
    pub textarea_ref: NodeRef<html::Textarea>
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::io::Cursor;

    use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
    use quick_xml::{Reader, Writer};

    use crate::constants::SPOILER_TAG;
    use crate::editor::get_styled_html_from_markdown;
    use crate::errors::AppError;

    pub async fn get_html_and_markdown_bodies(body: String, is_markdown: bool) -> Result<(String, Option<String>), AppError> {
        match is_markdown {
            true => Ok((
                get_styled_html_from_markdown(body.clone()).await?,
                Some(body),
            )),
            false => Ok((body, None)),
        }
    }

    pub fn style_html_user_content(user_content: &str) -> Result<String, AppError> {
        let mut reader = Reader::from_str(user_content);
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let mut elem = e.clone().into_owned();

                    match elem.name().as_ref() {
                        b"h1" => elem.push_attribute(("class", "text-4xl my-2")),
                        b"h2" => elem.push_attribute(("class", "text-2xl my-2")),
                        b"h3" => elem.push_attribute(("class", "text-xl my-2")),
                        b"a" => elem.push_attribute(("class", "link text-primary")),
                        b"ul" => elem.push_attribute(("class", "list-inside list-disc")),
                        b"ol" => elem.push_attribute(("class", "list-inside list-decimal")),
                        b"code" => {
                            elem.push_attribute(("class", "rounded-md bg-black p-0.5 px-1 mx-0.5"))
                        }
                        b"table" => elem.push_attribute(("class", "table")),
                        b"blockquote" => elem.push_attribute((
                            "class",
                            "w-fit p-1 my-1 border-s-4 rounded-sm border-slate-400 bg-slate-600",
                        )),
                        _ => (),
                    }

                    // writes the event to the writer
                    writer.write_event(Event::Start(elem))?;
                }
                Ok(Event::Empty(e)) => {
                    let mut elem = e.clone().into_owned();

                    match elem.name().as_ref() {
                        b"hr" => elem.push_attribute(("class", "my-2")),
                        _ => (),
                    }
                    // writes the event to the writer
                    writer.write_event(Event::Empty(elem))?;
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap().into_owned();
                    let spoiler_split_text = text.split(SPOILER_TAG);
                    let mut is_current_text_spoiler = None;
                    for text in spoiler_split_text {
                        let is_spoiler_text = is_current_text_spoiler.unwrap_or_default();
                        if !text.is_empty() {
                            if is_spoiler_text {
                                // Add label to encapsulate spoilers and a checkbox to toggle them
                                let label = BytesStart::new("label");
                                writer.write_event(Event::Start(label))?;
                                // Add invisible checkbox to toggle spoilers
                                let mut checkbox_elem = BytesStart::new("input");
                                checkbox_elem.push_attribute(("type", "checkbox"));
                                checkbox_elem.push_attribute(("class", "spoiler-checkbox hidden"));
                                writer.write_event(Event::Empty(checkbox_elem))?;

                                let mut span = BytesStart::new("span");
                                span.push_attribute(("class", "transition-all duration-300 ease-in-out rounded-md bg-white p-0.5 px-1 mx-0.5 text-white spoiler-text"));
                                writer.write_event(Event::Start(span))?;

                                writer.write_event(Event::Text(BytesText::new(text.trim())))?;

                                let span_end = BytesEnd::new("span");
                                writer.write_event(Event::End(span_end))?;

                                let label_end = BytesEnd::new("label");
                                writer.write_event(Event::End(label_end))?;
                            } else {
                                writer.write_event(Event::Text(BytesText::new(text)))?;
                            }
                        }
                        is_current_text_spoiler = Some(!is_spoiler_text);
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
                    return Err(AppError::from(e));
                }
            }
        }

        let styled_html_output = String::from_utf8(writer.into_inner().into_inner())?;
        log::debug!("Styled html: {styled_html_output}");
        Ok(styled_html_output)
    }
}

#[server]
pub async fn get_styled_html_from_markdown(
    markdown_input: String,
) -> Result<String, ServerFnError<AppError>> {
    let html_from_markdown =
        markdown::to_html_with_options(markdown_input.as_str(), &markdown::Options::gfm())
            .or_else(|e| Err(AppError::new(e)))?;
    log::debug!("Markdown as html: {html_from_markdown}");

    // Add styling, will be done by parsing the html which is a bit ugly. Would be better
    // if the styling could be added directly when generating the html from markdown
    let styled_html_output = ssr::style_html_user_content(html_from_markdown.as_str())?;
    Ok(styled_html_output)
}

/// Component for a textarea that can render simple text
#[component]
pub fn FormTextEditor(
    /// name of the textarea in the form that contains this component, must correspond to the parameter of the associated server function
    name: &'static str,
    /// name of the hidden checkbox indicating whether markdown mode is enabled, must correspond to the parameter of the associated server function
    placeholder: &'static str,
    /// Signals and node ref to control textarea content
    data: TextareaData,
    /// Additional css classes
    #[prop(default = "w-full")]
    class: &'static str,
) -> impl IntoView {
    let class = format!("group max-w-full p-2 border border-primary rounded-xs bg-base-100 {class}");

    view! {
        <div class=class>
            <div class="w-full rounded-t-lg">
                <label for=name class="sr-only">
                    {placeholder}
                </label>
                <textarea
                    id=name
                    name=name
                    placeholder=placeholder
                    class="w-full bg-base-100 resize-none box-border outline-hidden border-none"
                    on:input=move |ev| {
                        data.set_content.set(event_target_value(&ev));
                    }
                    node_ref=data.textarea_ref
                >
                    {data.content}
                </textarea>
            </div>
        </div>
    }.into_any()
}

/// Component for a textarea that can render markdown
#[component]
pub fn FormMarkdownEditor(
    /// name of the textarea in the form that contains this component, must correspond to the parameter of the associated server function
    name: &'static str,
    /// name of the hidden checkbox indicating whether markdown mode is enabled, must correspond to the parameter of the associated server function
    is_markdown_name: &'static str,
    /// Placeholder for the textarea
    placeholder: &'static str,
    /// Signals and node ref to control textarea content
    data: TextareaData,
    /// Initial state for markdown rendering
    #[prop(default = false)]
    is_markdown: bool,
) -> impl IntoView {
    let is_markdown_mode = RwSignal::new(is_markdown);
    let is_markdown_mode_string = move || is_markdown_mode.get().to_string();
    let markdown_button_class = move || match is_markdown_mode.get() {
        true => "h-full content-center p-2 rounded-md bg-success",
        false => "h-full content-center p-2 rounded-md hover:bg-base-200",
    };

    // Debounced version of the signals to avoid too many requests, also for is_markdown_mode so that
    // we wait for the debounced
    let content_debounced: Signal<String> = signal_debounced(data.content, 500.0);
    let is_md_mode_debounced: Signal<bool> = signal_debounced(is_markdown_mode, 500.0);

    let render_markdown_resource = Resource::new(
        move || (is_md_mode_debounced.get(), content_debounced.get()),
        move |(is_markdown_mode, markdown_content)| async move {
            if is_markdown_mode {
                get_styled_html_from_markdown(markdown_content).await
            } else {
                Ok(String::default())
            }
        },
    );

    view! {
        <div class="flex flex-col gap-2">
            <div class="group w-full max-w-full p-2 border border-primary rounded-xs">
                <div class="w-full mb-1 rounded-t-lg">
                    <label for=name class="sr-only">
                        {placeholder}
                    </label>
                    <textarea
                        id=name
                        name=name
                        placeholder=placeholder
                        class="w-full resize-none box-border bg-base-100 p-1 rounded-xs outline-hidden"
                        autofocus
                        on:input=move |ev| {
                            data.set_content.set(event_target_value(&ev));
                        }
                        node_ref=data.textarea_ref
                    >
                        {data.content}
                    </textarea>
                </div>
                <div class="flex justify-between items-center">
                    <div class="flex bg-base-300 rounded-md">
                        <label>
                            <input
                                type="text"
                                class="hidden"
                                name=is_markdown_name
                                value=is_markdown_mode_string
                                on:click=move |_| is_markdown_mode.update(|value| *value = !*value)
                            />
                            <div class=markdown_button_class>
                                <MarkdownIcon/>
                            </div>
                        </label>
                        <FormatButton format_type=FormatType::Bold data is_markdown_mode/>
                        <FormatButton format_type=FormatType::Italic data is_markdown_mode/>
                        <FormatButton format_type=FormatType::Strikethrough data is_markdown_mode/>
                        <FormatButton format_type=FormatType::Header1 data is_markdown_mode/>
                        <FormatButton format_type=FormatType::Header2 data is_markdown_mode/>
                        <FormatButton format_type=FormatType::List data is_markdown_mode/>
                        <FormatButton format_type=FormatType::NumberedList data is_markdown_mode/>
                        <FormatButton format_type=FormatType::CodeBlock data is_markdown_mode/>
                        <FormatButton format_type=FormatType::Spoiler data is_markdown_mode/>
                        <FormatButton format_type=FormatType::BlockQuote data is_markdown_mode/>
                        <FormatButton format_type=FormatType::Link data is_markdown_mode/>
                        <FormatButton format_type=FormatType::Image data is_markdown_mode/>
                    </div>
                    <div class="bg-base-300 rounded-full">
                        <HelpButton/>
                    </div>
                </div>
            </div>
            <Show when=is_markdown_mode>
                <TransitionUnpack resource=render_markdown_resource let:markdown_as_html>
                    <div class="w-full max-w-full min-h-24 max-h-96 overflow-auto overscroll-auto p-2 border border-primary rounded-xs bg-base-100 break-words"
                        inner_html={markdown_as_html.clone()}
                    />
                </TransitionUnpack>
            </Show>
        </div>
    }.into_any()
}

/// Component to format the selected text in the given textarea
#[component]
pub fn FormatButton(
    /// Signals and node ref to control textarea content
    data: TextareaData,
    /// signal indicating whether markdown rendering is activated
    is_markdown_mode: RwSignal<bool>,
    /// format operation of the button
    format_type: FormatType,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="p-2 rounded-md hover:bg-base-200"
            on:click=move |_| {
                if let Some(textarea_ref) = data.textarea_ref.get_untracked() {
                    let selection_start = textarea_ref.selection_start();
                    let selection_end = textarea_ref.selection_end();
                    match (selection_start, selection_end) {
                        (Ok(Some(selection_start)), Ok(Some(selection_end))) => {
                            let selection_start = selection_start as usize;
                            let selection_end = selection_end as usize;
                            format_textarea_content(
                                &mut data.set_content.write(),
                                selection_start,
                                selection_end,
                                format_type,
                            );
                            textarea_ref.set_value(&*data.content.read_untracked());
                            if !is_markdown_mode.get_untracked() {
                                is_markdown_mode.set(true);
                            }
                        },
                        _ => log::debug!("Failed to get textarea selections."),
                    };
                }
            }
        >
        {
            match format_type {
                FormatType::Bold => view!{ <BoldIcon/> }.into_any(),
                FormatType::Italic => view!{ <ItalicIcon/> }.into_any(),
                FormatType::Strikethrough => view!{ <StrikethroughIcon/> }.into_any(),
                FormatType::Header1 => view!{ <Header1Icon/> }.into_any(),
                FormatType::Header2 => view!{ <Header2Icon/> }.into_any(),
                FormatType::List => view!{ <ListBulletIcon/> }.into_any(),
                FormatType::NumberedList => view!{ <ListNumberIcon/> }.into_any(),
                FormatType::CodeBlock => view!{ <CodeBlockIcon/> }.into_any(),
                FormatType::Spoiler => view!{ <SpoilerIcon/> }.into_any(),
                FormatType::BlockQuote => view!{ <QuoteIcon/> }.into_any(),
                FormatType::Link => view!{ <LinkIcon/> }.into_any(),
                FormatType::Image => view!{ <ImageIcon/> }.into_any(),
            }
        }
        </button>
    }.into_any()
}

/// Component to render editor's help button
#[component]
pub fn HelpButton() -> impl IntoView {
    let show_help = RwSignal::new(false);
    let modal_ref = NodeRef::<html::Div>::new();
    let _ = on_click_outside(modal_ref, move |_| show_help.set(false));

    view! {
        <div class="relative inline-block z-20">
            <Show when=show_help>
                <div class="relative z-30">
                    <div
                        class="absolute bottom-0 right-0 z-40 origin-top-right mb-1 -mr-1 p-2 w-128 bg-base-200/90 rounded-sm"
                        node_ref=modal_ref
                    >
                        <div class="relative flex flex-col gap-2 leading-snug text-justify text-sm">
                            <p>
                                "To add formatting to your content, the 'Markdown mode' must be activated with the following button: "
                                <span class="inline-flex align-bottom w-fit p-1 mt-1 rounded-md bg-base-content/20"><MarkdownIcon/></span>
                            </p>
                            <p>
                                "When the 'Markdown mode' is activated, your input will be parsed using "
                                <a class="link text-primary" href="https://github.github.com/gfm/" >"GitHub Flavored Markdown"</a>
                                r#" (with the addition of Spoilers) and a preview of your content will be displayed.
                                   Quick-format buttons are also available so that you don't need to remember the GFM syntax!
                                   Finally, 'Spoiler' formatting can be generated by adding '||' on both side of your text or
                                   by selecting it and using the 'Spoiler' quick format button: "#
                                <span class="inline-flex align-bottom w-fit p-1 mt-1 rounded-md bg-base-content/20"><SpoilerIcon/></span>
                            </p>
                        </div>
                    </div>
                </div>
            </Show>
            <button
                type="button"
                class="p-2 rounded-full hover:bg-base-200"
                on:click=move |_| show_help.set(true)
            >
                <HelpIcon/>
            </button>
        </div>
    }
}

fn format_textarea_content(
    content: &mut String,
    mut selection_start: usize,
    mut selection_end: usize,
    format_type: FormatType,
) {
    let selected_content = &content[selection_start..selection_end];
    let leading_whitespace = selected_content
        .chars()
        .take_while(|ch| ch.is_whitespace())
        .count();
    let ending_whitespace = selected_content
        .chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count();

    selection_start += leading_whitespace;
    selection_end -= ending_whitespace;

    match format_type {
        FormatType::Bold => {
            content.insert_str(selection_end, "**");
            content.insert_str(selection_start, "**");
        }
        FormatType::Italic => {
            content.insert_str(selection_end, "*");
            content.insert_str(selection_start, "*");
        }
        FormatType::Strikethrough => {
            content.insert_str(selection_end, "~~");
            content.insert_str(selection_start, "~~");
        }
        FormatType::Header1 => {
            content.insert_str(get_line_start_for_position(content, selection_start), "# ");
        }
        FormatType::Header2 => {
            content.insert_str(get_line_start_for_position(content, selection_start), "## ");
        }
        FormatType::List => {
            content.insert_str(get_line_start_for_position(content, selection_start), "* ");
        }
        FormatType::NumberedList => {
            content.insert_str(get_line_start_for_position(content, selection_start), "1. ");
        }
        FormatType::CodeBlock => {
            content.insert_str(selection_end, "```");
            content.insert_str(selection_start, "```");
        }
        FormatType::Spoiler => {
            content.insert_str(selection_end, SPOILER_TAG);
            content.insert_str(selection_start, SPOILER_TAG);
        }
        FormatType::BlockQuote => {
            content.insert_str(get_line_start_for_position(content, selection_start), "> ");
        }
        FormatType::Link => {
            content.insert_str(
                get_line_start_for_position(content, selection_start),
                "[link text](https://www.your_link.com)",
            );
        }
        FormatType::Image => {
            content.insert_str(
                get_line_start_for_position(content, selection_start),
                "![](https://image_url.png)",
            );
        }
    };
}

/// Given the input String, returns the starting byte index of the line containing the [position] byte index.
fn get_line_start_for_position(string: &String, position: usize) -> usize {
    match string[..position].rfind('\n') {
        Some(line_start) => line_start + 1,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use leptos::prelude::ServerFnError;

    use crate::editor::ssr::get_html_and_markdown_bodies;
    use crate::editor::{get_styled_html_from_markdown, ssr::style_html_user_content};

    #[tokio::test]
    async fn test_get_html_and_markdown_bodies() -> Result<(), ServerFnError> {
        let text_body = "hello world";
        let markdown_body = "#this is a header";
        
        let (html_text_body, markdown_text_body) = get_html_and_markdown_bodies(
            text_body.to_string(), 
            false
        ).await.expect("Should get text body");
        assert_eq!(html_text_body, text_body);
        assert_eq!(markdown_text_body, None);

        let (html_markdown_body, markdown_markdown_body) = get_html_and_markdown_bodies(
            markdown_body.to_string(), 
            true
        ).await.expect("Should get text body");
        assert_eq!(html_markdown_body, get_styled_html_from_markdown(markdown_body.to_string()).await.expect("Should get html body"));
        assert_eq!(markdown_markdown_body.as_deref(), Some(markdown_body));
        
        Ok(())
    }

    #[test]
    fn test_style_html_user_content() -> Result<(), ServerFnError> {
        assert_eq!(
            style_html_user_content("<h1></h1>")?,
            r#"<h1 class="text-4xl my-2"></h1>"#
        );
        assert_eq!(
            style_html_user_content("<h2></h2>")?,
            r#"<h2 class="text-2xl my-2"></h2>"#
        );
        assert_eq!(
            style_html_user_content("<h3></h3>")?,
            r#"<h3 class="text-xl my-2"></h3>"#
        );
        assert_eq!(
            style_html_user_content("<a></a>")?,
            r#"<a class="link text-primary"></a>"#
        );
        assert_eq!(
            style_html_user_content("<ul></ul>")?,
            r#"<ul class="list-inside list-disc"></ul>"#
        );
        assert_eq!(
            style_html_user_content("<ol></ol>")?,
            r#"<ol class="list-inside list-decimal"></ol>"#
        );
        assert_eq!(
            style_html_user_content("<code></code>")?,
            r#"<code class="rounded-md bg-black p-0.5 px-1 mx-0.5"></code>"#
        );
        assert_eq!(
            style_html_user_content("<table></table>")?,
            r#"<table class="table"></table>"#
        );
        assert_eq!(
            style_html_user_content("<blockquote></blockquote>")?,
            r#"<blockquote class="w-fit p-1 my-1 border-s-4 rounded-sm border-slate-400 bg-slate-600"></blockquote>"#
        );
        assert_eq!(style_html_user_content("<hr/>")?, r#"<hr class="my-2"/>"#);
        assert_eq!(
            style_html_user_content("<p>Test, || This is a spoiler || this is not a spoiler</p>")?,
            r#"<p>Test, <label><input type="checkbox" class="spoiler-checkbox hidden"/><span class="transition-all duration-300 ease-in-out rounded-md bg-white p-0.5 px-1 mx-0.5 text-white spoiler-text">This is a spoiler</span></label> this is not a spoiler</p>"#
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_styled_html_from_markdown() -> Result<(), ServerFnError> {
        let markdown = indoc! {r#"
            # Here is a comment with markdown
            ## header 2
            ### header 3
            #### header 4
            ---
        "#};
        let expected_html = indoc! {r#"
            <h1 class="text-4xl my-2">Here is a comment with markdown</h1>
            <h2 class="text-2xl my-2">header 2</h2>
            <h3 class="text-xl my-2">header 3</h3>
            <h4>header 4</h4>
            <hr  class="my-2"/>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        let markdown = indoc! {r#"
            `code blocks`
        "#};
        let expected_html = indoc! {r#"
            <p><code class="rounded-md bg-black p-0.5 px-1 mx-0.5">code blocks</code></p>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        let markdown = indoc! {r#"
            || Spoilers ||
        "#};
        let expected_html = indoc! {r#"
            <p><label><input type="checkbox" class="spoiler-checkbox hidden"/><span class="transition-all duration-300 ease-in-out rounded-md bg-white p-0.5 px-1 mx-0.5 text-white spoiler-text">Spoilers</span></label></p>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        let markdown = indoc! {r#"
            **bold**, *italic*, combined emphasis with **asterisks and _underscores_**.
        "#};
        let expected_html = indoc! {r#"
            <p><strong>bold</strong>, <em>italic</em>, combined emphasis with <strong>asterisks and <em>underscores</em></strong>.</p>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        let markdown = indoc! {r#"
            Strikethrough uses two tildes. ~~Scratch this.~~
        "#};
        let expected_html = indoc! {r#"
            <p>Strikethrough uses two tildes. <del>Scratch this.</del></p>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        let markdown = indoc! {r#"
            > We can also do blockquotes
        "#};
        let expected_html = indoc! {r#"
            <blockquote class="w-fit p-1 my-1 border-s-4 rounded-sm border-slate-400 bg-slate-600">
            <p>We can also do blockquotes</p>
            </blockquote>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        let markdown = indoc! {r#"
            1. lists
            2. with numbers

            * lists
            * without numbers
            * and as many elements as we want
        "#};
        let expected_html = indoc! {r#"
            <ol class="list-inside list-decimal">
            <li>lists</li>
            <li>with numbers</li>
            </ol>
            <ul class="list-inside list-disc">
            <li>lists</li>
            <li>without numbers</li>
            <li>and as many elements as we want</li>
            </ul>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        let markdown = indoc! {r#"
            \
            Also, a bit more work is needed to add an empty line.
        "#};
        let expected_html = indoc! {r#"
            <p><br />
            Also, a bit more work is needed to add an empty line.</p>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        let markdown = indoc! {r#"
            Finally, we can add links [link text](https://www.example.com), images ![alt text](https://github.com/adam-p/markdown-here/raw/master/src/common/images/icon48.png "Logo Title Text 1")
        "#};
        let expected_html = indoc! {r#"
            <p>Finally, we can add links <a href="https://www.example.com" class="link text-primary">link text</a>, images <img src="https://github.com/adam-p/markdown-here/raw/master/src/common/images/icon48.png" alt="alt text" title="Logo Title Text 1" /></p>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        let markdown = indoc! {r#"
            | Tables        | Are           | Cool  |
            | ------------- |:-------------:| -----:|
            | col 3 is      | right-aligned | $1600 |
            | col 2 is      | centered      |   $12 |
            | zebra stripes | are neat      |    $1 |
        "#};
        let expected_html = indoc! {r#"
            <table class="table">
            <thead>
            <tr>
            <th>Tables</th>
            <th align="center">Are</th>
            <th align="right">Cool</th>
            </tr>
            </thead>
            <tbody>
            <tr>
            <td>col 3 is</td>
            <td align="center">right-aligned</td>
            <td align="right">$1600</td>
            </tr>
            <tr>
            <td>col 2 is</td>
            <td align="center">centered</td>
            <td align="right">$12</td>
            </tr>
            <tr>
            <td>zebra stripes</td>
            <td align="center">are neat</td>
            <td align="right">$1</td>
            </tr>
            </tbody>
            </table>
        "#};
        assert_eq!(
            get_styled_html_from_markdown(markdown.to_string()).await?,
            expected_html
        );

        Ok(())
    }
}
