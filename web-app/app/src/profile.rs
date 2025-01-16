use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos_router::params::ParamsMap;

pub const USER_ROUTE_PREFIX: &str = "/users";
pub const USER_ROUTE_PARAM_NAME: &str = "username";

/// Displays a user's profile
#[component]
pub fn UserProfile() -> impl IntoView {
    let params = use_params_map();
    let username = get_username_memo(params);
    view! {
        <div>{move || username.get()}</div>
    }
}

pub fn get_profile_path(
    username: &str,
) -> String {
    format!("{USER_ROUTE_PREFIX}/{username}")
}

/// Get a memo returning the last valid user id from the url. Used to avoid triggering resources when leaving pages.
pub fn get_username_memo(params: Memo<ParamsMap>) -> Memo<String> {
    Memo::new(move |current_username: Option<&String>| {
        if let Some(new_username) = params.read().get_str(USER_ROUTE_PARAM_NAME) {
            new_username.to_string()
        } else {
            log::trace!("Could not find new user id, reuse current user id: {current_username:?}");
            current_username.cloned().unwrap_or_default()
        }
    })
}