use leptos::*;

#[component]
pub fn FormTextEditor(
    name: &'static str,
    placeholder: &'static str,
    #[prop(default = false)]
    minimize: bool,
) -> impl IntoView {
    let do_minimize = create_rw_signal(minimize);

    view! {
        /*<div class="collapse ">
            <input type="checkbox" />
            <div class="collapse-title text-xl font-medium">
                Click me to show/hide content
            </div>
            <div class="collapse-content">
                <p>hello</p>
            </div>
        </div>*/
        <div
            on:focusin=move |_| { do_minimize.update(|do_minimize: &mut bool| *do_minimize = false); }
            on:focusout=move |_| { do_minimize.update(|do_minimize: &mut bool| *do_minimize = minimize); }
            class="rounded-btn border border-primary w-full transition-all ease-in-out"
            class=("h-textarea_s", move || do_minimize())
            class=("h-textarea_m", move || !do_minimize())
        >
            <textarea
                name=name
                placeholder=placeholder
                class="textarea w-full h-full"
            />
        </div>
    }
}