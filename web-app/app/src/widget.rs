use leptos::html;
use leptos::prelude::*;
use strum::IntoEnumIterator;

use crate::constants::{
    SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR,
};
use crate::icons::{AuthorIcon, ClockIcon, EditTimeIcon, InternalErrorIcon, MaximizeIcon, MinimizeIcon, ModeratorIcon, };

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
pub fn AuthorWidget(author: String) -> impl IntoView {
    view! {
        <div class="flex px-1 gap-1.5 items-center text-sm">
            <AuthorIcon/>
            {author}
        </div>
    }.into_any()
}

/// Component to display the moderator of a post or comment
#[component]
pub fn ModeratorWidget(
    #[prop(into)]
    moderator: MaybeSignal<Option<String>>
) -> impl IntoView {
    let moderator = StoredValue::new(moderator);
    view! {
        <Show when=move || moderator.get_value().with(|moderator| moderator.is_some())>
            <div class="flex px-1 gap-1.5 items-center text-sm">
                <ModeratorIcon/>
                {
                    move || moderator.get_value().get().unwrap_or_default()
                }
            </div>
        </Show>
    }.into_any()
}

/// Component to display the creation time of a post
#[component]
pub fn TimeSinceWidget(
    #[prop(into)]
    timestamp: MaybeSignal<chrono::DateTime<chrono::Utc>>
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
    edit_timestamp: MaybeSignal<Option<chrono::DateTime<chrono::Utc>>>
) -> impl IntoView {
    view! {
        <Show when=move || edit_timestamp.with(|edit_timestamp| edit_timestamp.is_some())>
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

/// Component to render a server action's error
#[component]
pub fn ActionError<F: Fn() -> bool + Send + Sync + 'static>(
    /// functions returning whether the publish buttons should be disabled
    has_error: F,
) -> impl IntoView {
    view! {
        <Show when=has_error>
            <div class="alert alert-error flex justify-center">
                <InternalErrorIcon/>
                <span>"Server error. Please reload the page and retry."</span>
            </div>
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