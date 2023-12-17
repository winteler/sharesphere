use leptos::*;

#[component]
pub fn FormTextEditor(
    name: &'static str,
    placeholder: &'static str,
    #[prop(default = false)]
    unfold_on_focus: bool,
) -> impl IntoView {

    /*let css_class = if unfold_on_focus {
        "textarea textarea-primary w-full h-textarea_s transition-all ease-in-out focus:h-textarea_m"
    } else {
        "textarea textarea-primary w-full h-textarea_m"
    };*/

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
        <textarea
            name=name
            placeholder=placeholder
            class="textarea textarea-primary w-full h-full"
        />
    }
}