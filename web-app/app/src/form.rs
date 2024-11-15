use leptos::prelude::*;

use crate::role::{AuthorizedShow, PermissionLevel};

/// Component for a boolean checkbox in a form
#[component]
pub fn FormCheckbox(
    /// Name of the input in the form that contains this component, must correspond to the parameter of the associated server function
    name: &'static str,
    #[prop(default = false)]
    value: bool,
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let is_checked = RwSignal::new(value);
    let is_checked_string = move || is_checked.get().to_string();
    view! {
        <div class=class>
            <input type="text" name=name value=is_checked_string class="hidden"/>
            <input type="checkbox" class="checkbox checkbox-primary" checked=is_checked on:click=move |_| is_checked.update(|value| *value = !*value)/>
        </div>
    }
}

/// Component for a boolean checkbox with a label in a form
#[component]
pub fn LabeledFormCheckbox(
    /// Name of the input in the form that contains this component, must correspond to the parameter of the associated server function
    name: &'static str,
    /// Label of the checkbox
    #[prop(default = "")]
    label: &'static str,
    #[prop(default = false)]
    value: bool,
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let is_checked = RwSignal::new(value);
    let is_checked_string = move || is_checked.get().to_string();
    view! {
        <div class=class>
            <input type="text" name=name value=is_checked_string class="hidden"/>
            <label class="cursor-pointer label p-0">
                <span class="label-text">{label}</span>
                <input type="checkbox" class="checkbox checkbox-primary" checked=is_checked on:click=move |_| is_checked.update(|value| *value = !*value)/>
            </label>
        </div>
    }
}

#[component]
pub fn IsPinnedCheckbox(
    #[prop(into)]
    forum_name: Signal<String>,
    #[prop(default = false)]
    value: bool,
) -> impl IntoView {
    view! {
        <AuthorizedShow forum_name permission_level=PermissionLevel::Moderate>
            <LabeledFormCheckbox name="is_pinned" label="Pinned" value/>
        </AuthorizedShow>
    }
}