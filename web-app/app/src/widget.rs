use leptos::ev::{Event, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{FileReader, FormData, HtmlFormElement, HtmlInputElement};
use strum::IntoEnumIterator;

use crate::app::GlobalState;
use crate::constants::{
    SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR,
};
use crate::errors::{AppError, ErrorDisplay};
use crate::icons::{ArrowUpIcon, AuthorIcon, ClockIcon, CommentIcon, EditTimeIcon, LoadingIcon, MaximizeIcon, MinimizeIcon, ModeratorAuthorIcon, ModeratorIcon, NsfwIcon, SaveIcon, SelfAuthorIcon, SpoilerIcon};

pub const SPHERE_NAME_PARAM: &str = "sphere_name";
pub const IMAGE_FILE_PARAM: &str = "image";

/// Component that displays its children in a modal dialog
#[component]
pub fn ModalDialog(
    class: &'static str,
    show_dialog: RwSignal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let dialog_class =
        move || format!("relative transform overflow-visible rounded transition-all {class}");
    view! {
        <Show when=show_dialog>
            <div
                class="relative z-10"
                aria-labelledby="modal-title"
                role="dialog"
                aria-modal="true"
            >
                <div class="fixed inset-0 bg-base-200 bg-opacity-75 transition-opacity"></div>
                <div class="fixed inset-0 z-10 w-screen overflow-auto">
                    <div class="flex min-h-full items-end justify-center items-center">
                        <div class=dialog_class>
                            {children()}
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }.into_any()
}

/// Component to create a dropdown based on a given strum::EnumIter
#[component]
pub fn EnumDropdown<I, T>(
    name: &'static str,
    enum_iter: I,
    _select_ref: NodeRef<html::Select>,
) -> impl IntoView
where
    I: IntoIterator<Item = T>,
    T: std::str::FromStr + Into<&'static str> + IntoEnumIterator
{
    view! {
        <select
            name=name
            class="select select-bordered w-fit"
            node_ref=_select_ref
        >
        {
            enum_iter.into_iter().map(|enum_val| view! {<option>{enum_val.into()}</option>}.into_any()).collect_view()
        }
        </select>
    }.into_any()
}

/// Component to display the author of a post or comment
#[component]
pub fn AuthorWidget(
    author: String,
    is_moderator: bool,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let author = StoredValue::new(author);

    view! {
        <div class="flex px-1 gap-1.5 items-center text-sm">
            { move || if is_moderator {
                    view! { <ModeratorAuthorIcon/> }.into_any()
                } else {
                    view! {
                        <Transition fallback=move || view! { <LoadingIcon/> }>
                        {
                            move || Suspend::new(async move {
                                match &state.user.await {
                                    Ok(Some(user)) if author.with_value(|author| *author == user.username) => view! { <SelfAuthorIcon/> }.into_any(),
                                    _ => view! { <AuthorIcon/> }.into_any(),
                                }
                            })
                        }
                        </Transition>
                    }.into_any()
                }
            }
            {author.get_value()}
        </div>
    }.into_any()
}

/// Component to display the number of comments in a post
#[component]
pub fn CommentCountWidget(
    count: i32,
) -> impl IntoView {
    view! {
        <div class="flex gap-1.5 items-center text-sm px-1">
            <CommentIcon/>
            {count}
        </div>
    }.into_any()
}

/// Component to display the moderator of a post or comment
#[component]
pub fn ModeratorWidget(
    #[prop(into)]
    moderator: Signal<Option<String>>
) -> impl IntoView {
    view! {
        <Show when=move || moderator.read().is_some()>
            <div class="flex px-1 gap-1.5 items-center text-sm">
                <ModeratorIcon/>
                {
                    move || moderator.get().unwrap_or_default()
                }
            </div>
        </Show>
    }.into_any()
}

/// Component to display a content's tags (spoiler, nsfw, ...)
#[component]
pub fn TagsWidget(
    is_nsfw: bool,
    is_spoiler: bool,
) -> impl IntoView {
    view! {
        <div class="flex gap-1">
        {
            match is_spoiler {
                true => Some(view! { <div class="h-fit w-fit px-1 py-0.5 bg-black rounded-full"><SpoilerIcon/></div> }),
                false => None
            }
        }
        {
            match is_nsfw {
                true => Some(view! { <NsfwIcon/>}),
                false => None
            }
        }
        </div>
    }
}

/// Component to display the creation time of a post
#[component]
pub fn TimeSinceWidget(
    #[prop(into)]
    timestamp: Signal<chrono::DateTime<chrono::Utc>>
) -> impl IntoView {
    view! {
        <div class="flex gap-1.5 items-center text-sm px-1">
            <ClockIcon/>
            {
                move || get_elapsed_time_string(timestamp.get())
            }
        </div>
    }.into_any()
}

/// Component to display the edit time of a post or comment
#[component]
pub fn TimeSinceEditWidget(
    #[prop(into)]
    edit_timestamp: Signal<Option<chrono::DateTime<chrono::Utc>>>
) -> impl IntoView {
    view! {
        <Show when=move || edit_timestamp.read().is_some()>
            <div class="flex gap-1.5 items-center text-sm px-1">
                <EditTimeIcon/>
                {
                    move || get_elapsed_time_string(edit_timestamp.get().unwrap())
                }
            </div>
        </Show>
    }
}

/// Component to display a minimize or maximize icon with transitions
#[component]
pub fn MinimizeMaximizeWidget(
    is_maximized: RwSignal<bool>
) -> impl IntoView {
    let invisible_class = "transition opacity-0 invisible h-0 w-0";
    let visible_class = "transition rotate-90 duration-300 opacity-100 visible";
    let minimize_class = move || match is_maximized.get() {
        true => visible_class,
        false => invisible_class,
    };
    let maximize_class = move || match is_maximized.get() {
        true => invisible_class,
        false => visible_class,
    };
    view! {
        <div>
            <div class=minimize_class>
                <MinimizeIcon/>
            </div>
            <div class=maximize_class>
                <MaximizeIcon/>
            </div>
        </div>
    }
}

/// Form to upload an image to the server
/// The form contains two inputs: a hidden sphere name and an image form
#[component]
pub fn SphereImageForm(
    #[prop(into)]
    sphere_name: Signal<String>,
    action: Action<FormData, Result<(), ServerFnError<AppError>>, LocalStorage>,
    #[prop(default = "max-h-80 max-w-full object-contain")]
    preview_class: &'static str,
) -> impl IntoView {
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
        let form_data = FormData::new_with_form(&target).unwrap();
        action.dispatch_local(form_data);
    };

    let preview_url = RwSignal::new(String::new());
    let on_file_change = move |ev| {
        let input: HtmlInputElement = event_target::<HtmlInputElement>(&ev);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                // Try to create a FileReader, returning early if it fails
                let reader = match FileReader::new() {
                    Ok(reader) => reader,
                    Err(_) => {
                        log::error!("Failed to create file reader.");
                        return
                    }, // Return early if FileReader creation fails
                };

                // Set up the onload callback for FileReader
                let preview_url_clone = preview_url.clone();
                let onload_callback = Closure::wrap(Box::new(move |e: Event| {
                    if let Some(reader) = e.target().and_then(|t| t.dyn_into::<FileReader>().ok()) {
                        if let Ok(Some(result)) = reader.result().and_then(|r| Ok(r.as_string())) {
                            preview_url_clone.set(result); // Update the preview URL
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                reader.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
                onload_callback.forget(); // Prevent the closure from being dropped

                // Start reading the file as a Data URL, returning early if it fails
                if let Err(e) = reader.read_as_data_url(&file) {
                    let error_message = e.as_string().unwrap_or_else(|| format!("{:?}", e));
                    log::error!("Error while getting preview of local image: {error_message}");
                };
            }
        }
    };

    view! {
        <form on:submit=on_submit class="flex flex-col gap-1">
            <input
                name=SPHERE_NAME_PARAM
                class="hidden"
                value=sphere_name
            />
            <input
                type="file"
                name=IMAGE_FILE_PARAM
                accept="image/*"
                class="file-input file-input-bordered file-input-primary w-full rounded-sm"
                on:change=on_file_change
            />
            <Show when=move || !preview_url.read().is_empty()>
                <img src=preview_url alt="Image Preview" class=preview_class/>
            </Show>
            <button
                type="submit"
                class="btn btn-secondary btn-sm p-1 self-end"
            >
                <SaveIcon/>
            </button>
            {move || {
                if action.pending().get()
                {
                    view! { <LoadingIcon/> }.into_any()
                } else {
                    match action.value().get()
                    {
                        Some(Ok(())) => {
                            if let Some(state) = use_context::<GlobalState>() {
                                state.sphere_reload_signal.update(|value| *value += 1);
                            }
                            ().into_any()
                        }
                        Some(Err(e)) => view! { <ErrorDisplay error=e.into()/> }.into_any(),
                        None => ().into_any()
                    }
                }
            }}
        </form>
    }
}

/// Component to render cancel and publish buttons for a modal Form
#[component]
pub fn ModalFormButtons(
    /// functions returning whether the publish buttons should be disabled
    #[prop(into)]
    disable_publish: Signal<bool>,
    /// signal to hide the form upon submitting or cancelling
    show_form: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex justify-between gap-2">
            <button
                type="button"
                class="btn btn-error"
                on:click=move |_| show_form.set(false)
            >
                "Cancel"
            </button>
            <button
                type="submit"
                class="btn btn-active btn-secondary"
                disabled=disable_publish
            >
                "Publish"
            </button>
        </div>
    }
}

/// Component to render cancel and publish buttons for a modal Form
#[component]
pub fn RotatingArrow(
    #[prop(into)]
    point_up: Signal<bool>,
    #[prop(default = "h-3 w-3")]
    class: &'static str,
) -> impl IntoView {
    let arrow_class = Signal::derive(move || match point_up.get() {
        true => format!("{class} transition duration-200"),
        false => format!("{class} transition duration-200 rotate-180"),
    });
    
    view! {
        <ArrowUpIcon class=arrow_class/>
    }
}

/// Component to render cancel and publish buttons for a modal Form
#[component]
pub fn Collapse<C>(
    #[prop(into)]
    title_view: ViewFnOnce,
    #[prop(default = true)]
    is_open: bool,
    children: TypedChildrenFn<C>,
) -> impl IntoView
where
    C : IntoView + 'static 
{
    let children = StoredValue::new(children.into_inner());
    let show_children = RwSignal::new(is_open);
    let children_class = move || match show_children.get() {
        true => "transition duration-500 opacity-100 visible",
        false => "opacity-0 invisible h-0",
    };
    
    view! {
        <div class="flex flex-col">
            <button
                class="p-1 rounded-md hover:bg-base-content/20"
                on:click=move |_| show_children.update(|value| *value = !*value)
            >
                <div class="flex justify-between items-center">
                    <div>{title_view.run()}</div>
                    <RotatingArrow point_up=show_children/>
                </div>
            </button>
            <div class=children_class>
            {
                children.with_value(|children| children())
            }
            </div>
        </div>
    }
}


/// Component to display a title with collapsable children
#[component]
pub fn TitleCollapse<C: IntoView + 'static>(
    #[prop(into)]
    title: String,
    #[prop(default = "text-xl font-semibold")]
    title_class: &'static str,
    #[prop(default = true)]
    is_open: bool,
    children: TypedChildrenFn<C>,
) -> impl IntoView {
    let children = StoredValue::new(children.into_inner());
    let show_children = RwSignal::new(is_open);
    let children_class = move || match show_children.get() {
        true => "transition duration-500 opacity-100 visible",
        false => "opacity-0 invisible h-0",
    };
    let arrow_class = Signal::derive(move || match show_children.get() {
        true => "h-3 w-3 transition duration-200",
        false => "h-3 w-3 transition duration-200 rotate-180",
    });
    view! {
        <div class="flex flex-col">
            <button
                class="p-1 rounded-md hover:bg-base-content/20"
                on:click=move |_| show_children.update(|value| *value = !*value)
            >
                <div class="flex justify-between items-center">
                    <div class=title_class>{title}</div>
                    <ArrowUpIcon class=arrow_class/>
                </div>
            </button>
            <div class=children_class>
            {
                children.with_value(|children| children())
            }
            </div>
        </div>
    }
}

fn get_elapsed_time_string(
    timestamp: chrono::DateTime<chrono::Utc>,
) -> String {
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