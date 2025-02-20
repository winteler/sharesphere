use leptos::either::Either;
use leptos::ev::{Event, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{FileReader, FormData, HtmlFormElement, HtmlInputElement};
use leptos_router::components::Form;
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_router::NavigateOptions;
use strum::IntoEnumIterator;

use crate::app::GlobalState;
use crate::auth::{LoginGuardedButton};
use crate::constants::{
    SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR,
};
use crate::error_template::ErrorTemplate;
use crate::errors::{AppError, ErrorDisplay};
use crate::icons::{ArrowUpIcon, AuthorIcon, ClockIcon, CommentIcon, EditTimeIcon, LoadingIcon, MaximizeIcon, MinimizeIcon, ModeratorAuthorIcon, ModeratorIcon, NsfwIcon, SaveIcon, SelfAuthorIcon, SpoilerIcon};
use crate::profile::get_profile_path;

pub const SPHERE_NAME_PARAM: &str = "sphere_name";
pub const IMAGE_FILE_PARAM: &str = "image";

pub trait ToView {
    fn to_view(self) -> AnyView;
}

/// Component that displays its children in a modal dialog
#[component]
pub fn ModalDialog(
    #[prop(default = "")]
    class: &'static str,
    show_dialog: RwSignal<bool>,
    children: ChildrenFn,
    #[prop(optional)]
    modal_ref: NodeRef<html::Div>,
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
                    <div class="flex min-h-full justify-center items-center">
                        <div class=dialog_class node_ref=modal_ref>
                            {children()}
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }.into_any()
}

/// Form to update query parameter `query_param` with the value `title` upon clicking
#[component]
fn QueryTab(
    query_param: &'static str,
    query_value: &'static str,
) -> impl IntoView {
    let query = use_query_map();
    let tab_class = move || match query.read().get(query_param).unwrap_or_default() == query_value {
        true => "w-full text-center p-1 bg-base-content/20 hover:bg-base-content/50",
        false => "w-full text-center p-1 hover:bg-base-content/50",
    };
    view! {
        <Form method="GET" action="">
            <input type="search" class="hidden" name=query_param value=query_value/>
            <button type="submit" class=tab_class>{query_value}</button>
        </Form>
    }
}

/// Component to display a QueryTab based on the input query_to_view_map
#[component]
fn QueryTabs<I, T>(
    query_param: &'static str,
    query_enum_iter: I,
) -> impl IntoView
where
    I: IntoIterator<Item = T>,
    T: std::str::FromStr + Into<&'static str> + IntoEnumIterator
{
    view! {
        <div class="w-full grid grid-flow-col justify-stretch divide-x divide-base-content/20 border border-1 border-base-content/20">
        {
            query_enum_iter.into_iter().map(|enum_value| view! {
                <QueryTab query_param query_value=enum_value.into()/>
            }.into_any()).collect_view()
        }
        </div>
    }.into_any()
}

/// Component to display the view of the enum selected by the query parameter `query_param`
#[component]
fn QueryShow<I, T>(
    query_param: &'static str,
    query_enum_iter: I,
    #[prop(optional, into)]
    default_view: ViewFn,
) -> impl IntoView
where
    I: IntoIterator<Item = T> + Clone + Send + Sync + 'static,
    T: std::str::FromStr + Into<&'static str> + Copy + IntoEnumIterator + ToView
{
    let query = use_query_map();
    view! {
        {
            move || match &query_enum_iter.clone().into_iter().find(
                |query_value| Into::<&str>::into(*query_value) == query.read().get(query_param).unwrap_or_default()
            ) {
                Some(query_value) => Either::Left(query_value.to_view()),
                None => Either::Right(default_view.run()),
            }
        }
    }.into_any()
}

/// Component to display tabs based on the `query_enum_iter` and upon clicking them, update
/// the query parameter `query_param` with the enum value and display the view using the ToView trait
#[component]
pub fn EnumQueryTabs<I, T>(
    query_param: &'static str,
    query_enum_iter: I,
    #[prop(optional, into)]
    default_view: ViewFn,
) -> impl IntoView
where
    I: IntoIterator<Item = T> + Clone + Send + Sync + 'static,
    T: std::str::FromStr + Into<&'static str> + Copy + IntoEnumIterator + ToView
{
    view! {
        <div class="flex flex-col gap-2 pt-2 px-2 w-full max-2xl:items-center">
            <QueryTabs query_param query_enum_iter=query_enum_iter.clone()/>
            <QueryShow query_param query_enum_iter default_view/>
        </div>
    }
}

/// Tab component displaying the str corresponding to `value` and updating signal upon click.
/// Highlighted style when `signal` == `value`.
#[component]
fn SignalTab<T>(
    signal: RwSignal<T>,
    value: T,
) -> impl IntoView
where
    T: Copy + Into<&'static str> + PartialEq + Send + Sync + 'static
{
    let tab_class = move || match signal.read() == value {
        true => "w-full text-center p-1 bg-base-content/20 hover:bg-base-content/50",
        false => "w-full text-center p-1 hover:bg-base-content/50",
    };
    view! {
        <button class=tab_class on:click=move |_| signal.set(value)>
            {value.into()}
        </button>
    }
}

/// Component to display the view of the enum selected by the query parameter `query_param`
#[component]
fn EnumSignalShow<I, T>(
    enum_signal: RwSignal<T>,
    enum_iter: I,
) -> impl IntoView
where
    I: IntoIterator<Item = T> + Clone + Send + Sync + 'static,
    T: Copy + Default + IntoEnumIterator + PartialEq + ToView  + Send + Sync + 'static
{
    view! {
        {
            move || match &enum_iter.clone().into_iter().find(
                |enum_value| *enum_value == *enum_signal.read()
            ) {
                Some(enum_value) => Either::Left(enum_value.to_view()),
                None => Either::Right(T::default().to_view()),
            }
        }
    }.into_any()
}

/// Component to display views query parameter `query_param` with the value `title` upon clicking
#[component]
pub fn EnumSignalTabs<I, T>(
    enum_signal: RwSignal<T>,
    enum_iter: I,
) -> impl IntoView
where
    I: IntoIterator<Item = T> + Clone + Send + Sync + 'static,
    T: std::str::FromStr + Into<&'static str> + Copy + Default + IntoEnumIterator + PartialEq + ToView  + Send + Sync + 'static
{
    view! {
        <div class="flex flex-col gap-2 pt-2 px-2 w-full max-2xl:items-center">
            <div class="w-full grid grid-flow-col justify-stretch divide-x divide-base-content/20 border border-1 border-base-content/20">
                {
                    enum_iter.clone().into_iter().map(|enum_value| view! {
                        <SignalTab signal=enum_signal value=enum_value/>
                    }.into_any()).collect_view()
                }
            </div>
            <EnumSignalShow enum_signal enum_iter/>
        </div>
    }
}

/// Component to create a dropdown based on a given strum::EnumIter
#[component]
pub fn EnumDropdown<I, T>(
    name: &'static str,
    enum_iter: I,
    select_ref: NodeRef<html::Select>,
) -> impl IntoView
where
    I: IntoIterator<Item = T>,
    T: std::str::FromStr + Into<&'static str> + IntoEnumIterator
{
    view! {
        <select
            name=name
            class="select select-bordered w-fit"
            node_ref=select_ref
        >
        {
            enum_iter.into_iter().map(|enum_val| view! {<option>{enum_val.into()}</option>}.into_any()).collect_view()
        }
        </select>
    }.into_any()
}

/// Component to display a button opening a modal dialog if the user
/// is authenticated and redirecting to a login page otherwise
#[component]
pub fn LoginGuardedOpenModalButton<IV>(
    show_dialog: RwSignal<bool>,
    #[prop(into)]
    button_class: Signal<&'static str>,
    children: TypedChildrenFn<IV>,
) -> impl IntoView
where
    IV: IntoView + 'static
{
    view! {
        <LoginGuardedButton
            button_class
            button_action=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
            children
            attr:aria-expanded=move || show_dialog.get().to_string()
            attr:aria-haspopup="dialog"
        />
    }
}

/// Component to display the author of a post or comment
#[component]
pub fn AuthorWidget(
    author: String,
    is_moderator: bool,
) -> impl IntoView {
    let navigate = use_navigate();
    let state = expect_context::<GlobalState>();
    let author_profile_path = get_profile_path(&author);
    let aria_label = format!("Navigate to user {}'s profile with path {}", author, author_profile_path);
    let author = StoredValue::new(author);

    view! {
        <button
            class="flex p-1.5 rounded-full gap-1.5 items-center text-sm hover:bg-base-content/20"
            on:click=move |ev| {
                ev.prevent_default();
                navigate(author_profile_path.as_str(), NavigateOptions::default());
            }
            aria-label=aria_label
        >
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
        </button>
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

/// Component to display a loading indicator or error depending on the input signals
#[component]
pub fn LoadIndicators(
    load_error: Signal<Option<AppError>>,
    is_loading: Signal<bool>,
) -> impl IntoView {
    view! {
        <Show when=move || load_error.read().is_some()>
        {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(load_error.get().unwrap());
            view! {
                <li><div class="flex justify-start py-4"><ErrorTemplate outside_errors/></div></li>
            }
        }
        </Show>
        <Show when=is_loading>
            <li><LoadingIcon/></li>
        </Show>
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