use leptos::prelude::*;
use leptos::html::ElementType;
use leptos::wasm_bindgen::JsCast;
use web_sys::{HtmlElement};

pub fn is_fully_scrolled<NR>(
    node_ref: NodeRef<NR>,
) -> bool
where
    NR: ElementType,
    NR::Output: Clone + AsRef<HtmlElement> + JsCast + 'static,
{
    node_ref.get_untracked().map(|node_ref| {
        match node_ref.dyn_ref::<HtmlElement>() {
            Some(element) => element.scroll_top() + element.offset_height() >= element.scroll_height(),
            None => false,
        }
    }).unwrap_or_default()
}