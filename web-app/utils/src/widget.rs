use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos::web_sys::{FormData};
use leptos_router::components::Form;
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_router::NavigateOptions;
use leptos_use::on_click_outside;
use serde::de::DeserializeOwned;
use server_fn::client::Client;
use server_fn::codec::PostUrl;
use server_fn::request::ClientReq;
use server_fn::ServerFn;
use strum::IntoEnumIterator;

use crate::auth::{LoginGuardedButton};
use crate::constants::{
    SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR,
};
use crate::error_template::ErrorTemplate;
use crate::errors::{AppError};
use crate::form::LabeledSignalCheckbox;
use crate::icons::{ArrowUpIcon, AuthorIcon, ClockIcon, CommentIcon, DeleteIcon, DotMenuIcon, EditTimeIcon, LoadingIcon, MaximizeIcon, MinimizeIcon, ModeratorAuthorIcon, ModeratorIcon, NsfwIcon, PinnedIcon, SelfAuthorIcon, SpoilerIcon};
use crate::unpack::ActionError;
use crate::user::{get_profile_path, UserState};

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
        move || format!("relative transform overflow-visible rounded-sm transition-all {class}");
    view! {
        <Show when=show_dialog>
            <div
                class="relative z-20"
                aria-labelledby="modal-title"
                role="dialog"
                aria-modal="true"
            >
                <div class="fixed inset-0 bg-base-200/75 transition-opacity"></div>
                <div class="fixed inset-0 z-20 w-screen overflow-auto">
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
) -> impl IntoView
where
    I: IntoIterator<Item = T> + Clone + Send + Sync + 'static,
    T: std::str::FromStr + Into<&'static str> + Copy + Default + IntoEnumIterator + ToView
{
    let query = use_query_map();
    view! {
        {
            move || match &query_enum_iter.clone().into_iter().find(
                |query_value| Into::<&str>::into(*query_value) == query.read().get(query_param).unwrap_or_default()
            ) {
                Some(query_value) => Either::Left(query_value.to_view()),
                None => Either::Right(T::default().to_view()),
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
) -> impl IntoView
where
    I: IntoIterator<Item = T> + Clone + Send + Sync + 'static,
    T: std::str::FromStr + Into<&'static str> + Copy + Default + IntoEnumIterator + ToView
{
    view! {
        <div class="flex flex-col gap-4 pt-2 px-2 w-full h-full">
            <QueryTabs query_param query_enum_iter=query_enum_iter.clone()/>
            <QueryShow query_param query_enum_iter/>
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
            class="select w-fit"
            node_ref=select_ref
        >
        {
            enum_iter.into_iter().map(|enum_val| view! {<option>{enum_val.into()}</option>}.into_any()).collect_view()
        }
        </select>
    }.into_any()
}

/// Component to display a button with a three-dot icon opening a menu displaying the children of the component when clicked
#[component]
pub fn DotMenu<C: IntoView + 'static>(
    children: TypedChildrenFn<C>,
) -> impl IntoView {
    let show_menu = RwSignal::new(false);
    let dropdown_ref = NodeRef::<html::Div>::new();
    let _ = on_click_outside(dropdown_ref, move |_| show_menu.set(false));

    let button_class = move || match show_menu.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };

    let children = StoredValue::new(children.into_inner());

    view! {
        <div
            class="h-full relative"
            node_ref=dropdown_ref
        >
            <button
                class=button_class
                on:click= move |_| show_menu.update(|value| *value = !*value)
            >
                <DotMenuIcon/>
            </button>
            <Show when=show_menu>
                <div class="absolute z-10 origin-bottom-left">
                    <div class="bg-base-200 shadow-sm rounded-sm mt-1 p-1 w-fit">
                    {
                        children.with_value(|children| children())
                    }
                    </div>
                </div>
            </Show>
        </div>
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
    let user_state = expect_context::<UserState>();
    let author_profile_path = get_profile_path(&author);
    let aria_label = format!("Navigate to user {}'s profile with path {}", author, author_profile_path);
    let author = StoredValue::new(author);

    view! {
        <button
            class="flex p-1.5 rounded-full gap-1.5 items-center text-sm hover:bg-base-300"
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
                                match &user_state.user.await {
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

/// Component to conditionally display a pin icon
#[component]
pub fn IsPinnedWidget(
    #[prop(into)]
    is_pinned: Signal<bool>,
) -> impl IntoView {
    view! {
        { move || match is_pinned.get() {
            true => Some(view! { <div class="px-1"><PinnedIcon/></div>}),
            false => None
        }}
    }
}

/// Component to display a content's tags (spoiler, nsfw, ...)
#[component]
pub fn TagsWidget(
    is_nsfw: bool,
    is_spoiler: bool,
    #[prop(default = false)]
    is_pinned: bool,
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
        <IsPinnedWidget is_pinned/>
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

/// Component to render a delete button
#[component]
pub fn DeleteButton<A>(
    title: &'static str,
    id: i64,
    id_name: &'static str,
    author_id: i64,
    delete_action: ServerAction<A>
) -> impl IntoView
where
    A: DeserializeOwned
    + ServerFn<InputEncoding = PostUrl, Error = AppError>
    + Clone
    + Send
    + Sync
    + 'static,
    <<A::Client as Client<A::Error>>::Request as ClientReq<
        A::Error,
    >>::FormData: From<FormData>,
    A::Output: Send + Sync + 'static,
{
    let user_state = expect_context::<UserState>();
    let show_form = RwSignal::new(false);
    let show_button = move || match &(*user_state.user.read()) {
        Some(Ok(Some(user))) => user.user_id == author_id,
        _ => false,
    };
    let edit_button_class = move || match show_form.get() {
        true => "btn btn-circle btn-sm btn-error",
        false => "btn btn-circle btn-sm btn-ghost",
    };
    view! {
        <Show when=show_button>
            <div>
                <button
                    class=edit_button_class
                    aria-expanded=move || show_form.get().to_string()
                    aria-haspopup="dialog"
                    on:click=move |_| show_form.update(|show: &mut bool| *show = !*show)
                >
                    <DeleteIcon/>
                </button>
                <ModalDialog
                    class="w-full flex justify-center"
                    show_dialog=show_form
                >
                    <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-5 w-96">
                        <div class="text-center font-bold text-2xl">{title}</div>
                        <div class="text-center font-bold text-xl">"This cannot be undone."</div>
                        <ActionForm action=delete_action>
                            <input
                                name=id_name
                                class="hidden"
                                value=id
                            />
                            <ModalFormButtons
                                disable_publish=false
                                show_form
                            />
                        </ActionForm>
                        <ActionError action=delete_action.into()/>
                    </div>
                </ModalDialog>
            </div>
        </Show>
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
                class="btn btn-secondary"
                disabled=disable_publish
            >
                "Submit"
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
                class="p-1 rounded-md hover:bg-base-200"
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
        false => "opacity-0 invisible h-0 max-h-0 overflow-hidden",
    };
    let arrow_class = Signal::derive(move || match show_children.get() {
        true => "h-3 w-3 transition duration-200",
        false => "h-3 w-3 transition duration-200 rotate-180",
    });
    view! {
        <div class="flex flex-col shrink-0 relative">
            <button
                class="p-1 rounded-md hover:bg-base-200"
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

/// Component to display a checkbox to enable or disable NSFW results.
/// If the user is not logged in or has disabled NSFW in his settings, the checkbox is hidden and deactivated.
#[component]
pub fn NsfwCheckbox(
    show_nsfw: RwSignal<bool>,
    #[prop(default = "NSFW")]
    label: &'static str,
    #[prop(default = "pl-1")]
    class: &'static str,
) -> impl IntoView {
    let user_state = expect_context::<UserState>();
    view! {
        <Transition fallback=move || view! {  <LoadingIcon/> }>
        {
            move || Suspend::new(async move {
                match user_state.user.await {
                    Ok(Some(user)) if user.show_nsfw => Some(view! {
                        <LabeledSignalCheckbox label value=show_nsfw class=class/>
                    }),
                    _ => {
                        show_nsfw.set(false);
                        None
                    },
                }
            })
        }
        </Transition>
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