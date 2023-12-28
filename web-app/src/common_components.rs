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
    let is_empty = create_rw_signal(true);
    let hide_buttons = move || minimize && is_empty();

    view! {
        <div class="group w-full my-2 border border-primary rounded-lg bg-base-100">
            <div class="px-2 py-2 rounded-t-lg">
                <label for="comment" class="sr-only">"Your comment"</label>
                <textarea
                    id="comment"
                    name=name
                    placeholder=placeholder
                    class="w-full px-0 bg-base-100 outline-none border-none"
                    on:input=move |ev| {
                        is_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                    }
                />
            </div>

            <div
                class="flex justify-between px-2 pb-2"
                class:invisible=hide_buttons
                class=("group-focus-within:visible", move || minimize)
            >
                <div class="flex">
                    <button
                        type="button"
                        class="btn btn-ghost"
                        class:invisible=hide_buttons
                        class=("group-focus-within:visible", move || minimize)
                    >
                        <BoldIcon/>
                    </button>
                </div>
                <div>
                    <button
                        type:submit=move || with_publish_button
                        class="btn btn-active btn-secondary"
                        class:invisible=move || !with_publish_button || hide_buttons()
                        class=("group-focus-within:visible", move || minimize)
                        disabled=is_empty
                    >
                        "Publish"
                    </button>
                </div>
            </div>
        </div>
    }
}