#[cfg(feature = "ssr")]
use std::env;
use const_format::concatcp;
use leptos::prelude::{window, Memo, Read, RwSignal, Set, Update};
use leptos_router::params::ParamsMap;
use crate::constants::SITE_ROOT;
use crate::errors::AppError;

pub const APP_ORIGIN_ENV: &str = "APP_ORIGIN";
pub const PUBLISH_ROUTE: &str = "/publish";
pub const USER_ROUTE_PREFIX: &str = "/users";
pub const USER_ROUTE_PARAM_NAME: &str = "username";
pub const SPHERE_ROUTE_PREFIX: &str = "/spheres";
pub const SPHERE_ROUTE_PARAM_NAME: &str = "sphere_name";
pub const CREATE_SPHERE_SUFFIX: &str = "/sphere";
pub const CREATE_SPHERE_ROUTE: &str = concatcp!(PUBLISH_ROUTE, CREATE_SPHERE_SUFFIX);
pub const SATELLITE_ROUTE_PREFIX: &str = "/satellites";
pub const SATELLITE_ROUTE_PARAM_NAME: &str = "satellite_id";
pub const CREATE_POST_SUFFIX: &str = "/post";
pub const CREATE_POST_ROUTE: &str = concatcp!(PUBLISH_ROUTE, CREATE_POST_SUFFIX);
pub const CREATE_POST_SPHERE_QUERY_PARAM: &str = "sphere";
pub const POST_ROUTE_PREFIX: &str = "/posts";
pub const POST_ROUTE_PARAM_NAME: &str = "post_name";
pub const COMMENT_ID_QUERY_PARAM: &str = "comment_id";
pub const SEARCH_ROUTE: &str = "/search";
pub const SEARCH_TAB_QUERY_PARAM: &str = "type";
pub const TERMS_AND_CONDITIONS_ROUTE: &str = "/terms_and_conditions";
pub const PRIVACY_POLICY_ROUTE: &str = "/privacy_policy";

#[cfg(feature = "ssr")]
pub fn get_app_origin() -> Result<String, AppError> {
    Ok(env::var(APP_ORIGIN_ENV)?)
}

#[cfg(not(feature = "ssr"))]
pub fn get_app_origin() -> Result<String, AppError> {
    window().location().origin().map_err(|_| AppError::new("Failed to get base url"))
}

pub fn get_current_url(url: RwSignal<String>) {
    let url_str = window().location().href().unwrap_or(String::from(SITE_ROOT));
    log::debug!("Current url: {url_str}");
    url.update(|value| *value = url_str);
}

pub fn get_current_path(path: RwSignal<String>) {
    let path_str = window().location().pathname().unwrap_or(String::from(SITE_ROOT));
    log::debug!("Current path: {path_str}");
    path.update(|value | *value = path_str);
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

/// # Returns the path to a sphere given its name
///
/// ```
/// use sharesphere_utils::routes::get_sphere_path;
///
/// assert_eq!(get_sphere_path("test"), "/spheres/test");
/// ```
pub fn get_sphere_path(
    sphere_name: &str,
) -> String {
    format!("{SPHERE_ROUTE_PREFIX}/{sphere_name}")
}

/// # Extract the sphere name from the current path, if it exists
///
/// ```
/// use sharesphere_utils::routes::get_sphere_from_path;
///
/// assert_eq!(get_sphere_from_path("test"), None);
/// assert_eq!(get_sphere_from_path("/spheres/test"), Some(String::from("test")));
/// ```
pub fn get_sphere_from_path(path: &str) -> Option<String> {
    if path.starts_with(SPHERE_ROUTE_PREFIX) {
        let mut path_part_it = path.split("/");
        Some(String::from(path_part_it.nth(2).unwrap_or("")))
    } else {
        None
    }
}

pub fn get_sphere_name(sphere_name: RwSignal<String>) {
    let path = window().location().pathname().unwrap_or_default();
    sphere_name.update(|name| *name = get_sphere_from_path(&path).unwrap_or_default());
}

/// Get the current sphere name from the path. When the current path does not contain a sphere, returns the last valid sphere. Used to avoid sending a request when leaving a page
pub fn get_sphere_name_memo(params: Memo<ParamsMap>) -> Memo<String> {
    Memo::new(move |current_sphere_name: Option<&String>| {
        if let Some(new_sphere_name) = params.read().get_str(SPHERE_ROUTE_PARAM_NAME) {
            log::trace!("Current sphere name {current_sphere_name:?}, new sphere name: {new_sphere_name}");
            new_sphere_name.to_string()
        } else {
            log::trace!("No valid sphere name, keep current value: {current_sphere_name:?}");
            current_sphere_name.cloned().unwrap_or_default()
        }
    })
}

/// # Returns the path to a satellite given its id and sphere name
///
/// ```
/// use sharesphere_utils::routes::get_satellite_path;
///
/// assert_eq!(get_satellite_path("test", 1), "/spheres/test/satellites/1");
/// ```
pub fn get_satellite_path(
    sphere_name: &str,
    satellite_id: i64
) -> String {
    format!("{SPHERE_ROUTE_PREFIX}/{}{SATELLITE_ROUTE_PREFIX}/{}", sphere_name, satellite_id)
}

/// Get a memo returning the last valid satellite_id from the url. Used to avoid triggering resources when leaving pages
pub fn get_satellite_id_memo(params: Memo<ParamsMap>) -> Memo<i64> {
    Memo::new(move |current_satellite_id: Option<&i64>| {
        if let Some(new_satellite_id_str) = params.read().get_str(SATELLITE_ROUTE_PARAM_NAME) {
            if let Ok(new_satellite_id) = new_satellite_id_str.parse::<i64>() {
                log::trace!("Current satellite id: {current_satellite_id:?}, new satellite id: {new_satellite_id}");
                new_satellite_id
            } else {
                log::trace!("Could not parse new satellite id: {new_satellite_id_str}, reuse current satellite id: {current_satellite_id:?}");
                current_satellite_id.cloned().unwrap_or_default()
            }
        } else {
            log::trace!("Could not find new satellite id, reuse current satellite id: {current_satellite_id:?}");
            current_satellite_id.cloned().unwrap_or_default()
        }
    })
}

pub fn get_create_post_path(create_post_route: RwSignal<String>) {
    let path = window().location().pathname().unwrap_or_default();
    log::debug!("Current path: {path}");

    let current_sphere = get_sphere_from_path(&path);

    if let Some(sphere_name) = current_sphere {
        create_post_route.set(format!("{CREATE_POST_ROUTE}?{CREATE_POST_SPHERE_QUERY_PARAM}={sphere_name}"));
    } else {
        create_post_route.set(String::from(CREATE_POST_ROUTE));
    };
}

/// # Returns the path to a post given its id, sphere and optional satellite
///
/// ```
/// use sharesphere_utils::routes::get_post_path;
///
/// assert_eq!(get_post_path("test", None, 1), "/spheres/test/posts/1");
/// assert_eq!(get_post_path("test", Some(1), 2), "/spheres/test/satellites/1/posts/2");
/// ```
pub fn get_post_path(
    sphere_name: &str,
    satellite_id: Option<i64>,
    post_id: i64,
) -> String {
    match satellite_id {
        Some(satellite_id) => format!(
            "{SPHERE_ROUTE_PREFIX}/{sphere_name}{SATELLITE_ROUTE_PREFIX}/{satellite_id}{POST_ROUTE_PREFIX}/{}",
            post_id
        ),
        None => format!("{SPHERE_ROUTE_PREFIX}/{sphere_name}{POST_ROUTE_PREFIX}/{}", post_id)
    }
}

/// # Returns the url to a post given its id, sphere and optional satellite
///
/// ```
/// use sharesphere_utils::routes::{get_app_origin, get_post_link};
/// let origin = get_app_origin().unwrap_or_default();
/// assert_eq!(get_post_link("test", None, 1), format!("{origin}/spheres/test/posts/1"));
/// assert_eq!(get_post_link("test", Some(1), 2), format!("{origin}/spheres/test/satellites/1/posts/2"));
/// ```
pub fn get_post_link(
    sphere_name: &str,
    satellite_id: Option<i64>,
    post_id: i64,
) -> String {
    let base_url = get_app_origin().unwrap_or_default();
    let post_path = get_post_path(sphere_name, satellite_id, post_id);
    format!("{base_url}{post_path}")
}

/// # Returns the path to a comment given its id, post_id, sphere and optional satellite
///
/// ```
/// use sharesphere_utils::routes::get_comment_path;
///
/// assert_eq!(get_comment_path("test", None, 1, 2), "/spheres/test/posts/1?comment_id=2");
/// assert_eq!(get_comment_path("test", Some(1), 2, 3), "/spheres/test/satellites/1/posts/2?comment_id=3");
/// ```
pub fn get_comment_path(
    sphere_name: &str,
    satellite_id: Option<i64>,
    post_id: i64,
    comment_id: i64,
) -> String {
    match satellite_id {
        Some(satellite_id) => format!(
            "{SPHERE_ROUTE_PREFIX}/{sphere_name}{SATELLITE_ROUTE_PREFIX}/{satellite_id}{POST_ROUTE_PREFIX}/{post_id}?{COMMENT_ID_QUERY_PARAM}={comment_id}"
        ),
        None => format!("{SPHERE_ROUTE_PREFIX}/{sphere_name}{POST_ROUTE_PREFIX}/{post_id}?{COMMENT_ID_QUERY_PARAM}={comment_id}")
    }
}

/// # Returns the url to a comment given its id, post_id, sphere and optional satellite
///
/// ```
/// use sharesphere_utils::routes::{get_app_origin, get_comment_link};
/// let origin = get_app_origin().unwrap_or_default();
/// assert_eq!(get_comment_link("test", None, 1, 2), format!("{origin}/spheres/test/posts/1?comment_id=2"));
/// assert_eq!(get_comment_link("test", Some(1), 2, 3), format!("{origin}/spheres/test/satellites/1/posts/2?comment_id=3"));
/// ```
pub fn get_comment_link(
    sphere_name: &str,
    satellite_id: Option<i64>,
    post_id: i64,
    comment_id: i64,
) -> String {
    let base_url = get_app_origin().unwrap_or_default();
    let comment_path = get_comment_path(sphere_name, satellite_id, post_id, comment_id);
    format!("{base_url}{comment_path}")
}

/// Get a memo returning the last valid post id from the url. Used to avoid triggering resources when leaving pages
pub fn get_post_id_memo(params: Memo<ParamsMap>) -> Memo<i64> {
    Memo::new(move |current_post_id: Option<&i64>| {
        if let Some(new_post_id_string) = params.read().get_str(POST_ROUTE_PARAM_NAME) {
            if let Ok(new_post_id) = new_post_id_string.parse::<i64>() {
                log::trace!("Current post id: {current_post_id:?}, new post id: {new_post_id}");
                new_post_id
            } else {
                log::trace!("Could not parse new post id: {new_post_id_string}, reuse current post id: {current_post_id:?}");
                current_post_id.cloned().unwrap_or_default()
            }
        } else {
            log::trace!("Could not find new post id, reuse current post id: {current_post_id:?}");
            current_post_id.cloned().unwrap_or_default()
        }
    })
}