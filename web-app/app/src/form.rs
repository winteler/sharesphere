use leptos::prelude::*;

use crate::role::{AuthorizedShow, PermissionLevel};

/// Component for a boolean checkbox in a form
#[component]
pub fn FormCheckbox(
    /// Name of the input in the form that contains this component, must correspond to the parameter of the associated server function
    name: &'static str,
    is_checked: RwSignal<bool>,
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
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
    #[prop(optional)]
    label: &'static str,
    #[prop(optional)]
    value: bool,
    #[prop(optional)]
    disabled: bool,
    #[prop(optional)]
    class: &'static str,
) -> impl IntoView {
    let is_checked = RwSignal::new(value);
    let is_checked_string = move || is_checked.get().to_string();
    view! {
        <div class=class>
            <input type="text" name=name value=is_checked_string class="hidden"/>
            <label class="cursor-pointer flex justify-between">
                <span class="label">{label}</span>
                <input
                    type="checkbox"
                    class="checkbox checkbox-primary"
                    checked=is_checked
                    disabled=disabled
                    on:click=move |_| is_checked.update(|value| *value = !*value)
                />
            </label>
        </div>
    }
}

/// Component for a boolean checkbox with a label updating a signal
#[component]
pub fn LabeledSignalCheckbox(
    label: &'static str,
    value: RwSignal<bool>,
    #[prop(optional)]
    disabled: bool,
    #[prop(optional)]
    class: &'static str,
) -> impl IntoView {
    view! {
        <div class=class>
            <label class="cursor-pointer flex justify-between">
                <span class="label">{label}</span>
                <input
                    type="checkbox"
                    class="checkbox checkbox-primary"
                    checked=value
                    disabled=disabled
                    on:click=move |_| value.update(|value| *value = !*value)
                />
            </label>
        </div>
    }
}

#[component]
pub fn IsPinnedCheckbox(
    #[prop(into)]
    sphere_name: Signal<String>,
    #[prop(default = false)]
    value: bool,
) -> impl IntoView {
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Moderate>
            <LabeledFormCheckbox name="is_pinned" label="Pinned" value/>
        </AuthorizedShow>
    }
}