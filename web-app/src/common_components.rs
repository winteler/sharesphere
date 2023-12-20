use leptos::*;

use crate::icons::{BoldIcon};

#[component]
pub fn FormTextEditor(
    name: &'static str,
    placeholder: &'static str,
    #[prop(default = false)]
    minimize: bool,
    #[prop(default = false)]
    with_publish_button: bool,
) -> impl IntoView {
    let do_minimize = create_rw_signal(minimize);
    let is_empty = create_rw_signal(true);

    view! {
        <div
            on:focusin=move |_| { do_minimize.update(|do_minimize: &mut bool| *do_minimize = false); }
            on:focusout=move |_| { do_minimize.update(|do_minimize: &mut bool| *do_minimize = minimize && is_empty()); }
            class="flex flex-col rounded-btn border border-primary w-full transition-all ease-in-out"
            class=("h-textarea_s", move || do_minimize())
            class=("h-textarea_m", move || !do_minimize())
        >
            <textarea
                name=name
                placeholder=placeholder
                class="textarea w-full h-full"
                on:input=move |ev| {
                    is_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                }
            />
            <div class="flex flex-col justify-between">
                <div class="flex">
                    <button class="btn btn-ghost">
                        <BoldIcon/>
                    </button>
                </div>
                <div>
                    <button
                        type:submit=move || with_publish_button
                        class="btn btn-active btn-secondary"
                        class:hidden=move || !with_publish_button
                        disabled=is_empty
                    >
                        "Publish"
                    </button>
                </div>
            </div>
        </div>
    }
}