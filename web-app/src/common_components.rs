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
    let row_num = move || {
        if do_minimize() {
            1
        }
        else {
            4
        }
    };

    view! {
        <div
            on:focusin=move |_| { do_minimize.update(|do_minimize: &mut bool| *do_minimize = false); }
            on:focusout=move |_| { do_minimize.update(|do_minimize: &mut bool| *do_minimize = minimize && is_empty()); }
            class="w-full mb-4 border border-primary rounded-lg bg-base-100"
        >
            <div class="px-2 py-2 rounded-t-lg">
                <label for="comment" class="sr-only">"Your comment"</label>
                <textarea
                    id="comment"
                    name=name
                    placeholder=placeholder
                    rows=row_num
                    class="w-full px-0 bg-base-100 outline-none border-none"
                    on:input=move |ev| {
                        is_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                    }
                />
            </div>

            <div class="flex justify-between px-2 pb-2"
                 class:hidden=do_minimize
            >
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