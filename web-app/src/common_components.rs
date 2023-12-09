use leptos::*;

#[component]
pub fn FormTextEditor(
    name: &'static str,
    placeholder: &'static str,
    #[prop(default = false)]
    unfold_on_focus: bool,
) -> impl IntoView {

    let css_class = if unfold_on_focus {
        "textarea textarea-primary w-full h-textarea_s transition-all ease-in-out focus:h-textarea_m"
    } else {
        "textarea textarea-primary w-full h-textarea_m"
    };

    view! {
        <textarea
            name=name
            placeholder=placeholder
            class=css_class
        />
    }
}