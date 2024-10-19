use leptos::prelude::*;

/// Component for a boolean checkbox in a form
#[component]
pub fn FormCheckbox(
    /// Name of the input in the form that contains this component, must correspond to the parameter of the associated server function
    name: &'static str,
    /// Label of the checkbox
    label: &'static str,
) -> impl IntoView {
    let is_checked = RwSignal::new(false);
    let is_checked_string = move || is_checked.get().to_string();
    view! {
        <div class="form-control">
            <input type="text" name=name value=is_checked_string class="hidden"/>
            <label class="cursor-pointer label p-0">
                <span class="label-text">{label}</span>
                <input type="checkbox" class="checkbox checkbox-primary" checked=is_checked on:click=move |_| is_checked.update(|value| *value = !*value)/>
            </label>
        </div>
    }
}