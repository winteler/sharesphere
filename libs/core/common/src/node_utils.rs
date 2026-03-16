use leptos::prelude::*;
use leptos::html::{ElementType, Input};
use leptos::wasm_bindgen::JsCast;
use web_sys::{HtmlElement};

pub fn has_reached_scroll_load_threshold<NR>(
    node_ref: NodeRef<NR>,
) -> bool
where
    NR: ElementType,
    NR::Output: Clone + AsRef<HtmlElement> + JsCast + 'static,
{
    node_ref.get().map(|node_ref| {
        match node_ref.dyn_ref::<HtmlElement>() {
            Some(element) => {
                let scroll_height = element.scroll_top() + element.offset_height();
                scroll_height >= element.scroll_height() - 2 * element.offset_height()
            },
            None => false,
        }
    }).unwrap_or_default()
}

pub fn set_checkbox(is_checked: bool, input_ref: NodeRef<Input>) {
    if let Some(input_ref) = input_ref.get() {
        input_ref.set_checked(is_checked);
    }
}