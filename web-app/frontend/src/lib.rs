use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

use app::app::*;

#[wasm_bindgen]
pub fn hydrate() {
    // initializes logging using the `log` crate
    _ = console_log::init_with_level(log::Level::Info);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
