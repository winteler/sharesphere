pub mod app;
pub mod auth;
pub mod comment;
pub mod constants;
pub mod post;
pub mod errors;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fallback;
pub mod footer;
pub mod forum;
pub mod icons;
pub mod navigation_bar;
pub mod score;
pub mod sidebar;
pub mod widget;
#[cfg(feature = "ssr")]
pub mod state;


#[cfg(feature = "hydrate")]
use crate::app::App;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
