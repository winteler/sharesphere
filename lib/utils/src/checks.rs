use const_format::formatcp;
use url::Url;
use validator::ValidationError;
use crate::constants::{MAX_SPHERE_NAME_LENGTH, MAX_USERNAME_LENGTH};
use crate::errors::AppError;
use crate::routes::get_app_origin;

pub fn check_string_length(
    input: &str,
    input_name: &str,
    max_length: usize,
) -> Result<(), AppError> {
    match input.len() > max_length {
        true => Err(AppError::new(format!("{input_name} exceeds the maximum length {max_length}."))),
        false => Ok(()),
    }
}

/// # Returns whether a sphere name is valid.
///
/// # Valid sphere names contain only ascii alphanumeric characters, '-', '_' and have a maximum length of `MAX_SPHERE_NAME_LENGTH`
///
/// ```
/// use sharesphere_utils::checks::{check_sphere_name};
/// use sharesphere_utils::constants::MAX_SPHERE_NAME_LENGTH;
/// use sharesphere_utils::errors::AppError;
///
/// assert!(check_sphere_name("-Abc123_").is_ok());
/// assert!(check_sphere_name(" name").is_err());
/// assert!(check_sphere_name("name%").is_err());
/// assert!(check_sphere_name(&"a".repeat(MAX_SPHERE_NAME_LENGTH)).is_ok());
/// assert!(check_sphere_name(&"a".repeat(MAX_SPHERE_NAME_LENGTH + 1)).is_err());
/// ```
pub fn check_sphere_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        Err(ValidationError::new("Sphere name cannot be empty."))
    } else if !name.chars().all(move |c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        Err(ValidationError::new("Sphere name can only contain alphanumeric characters, dashes and underscores."))
    } else if name.len() > MAX_SPHERE_NAME_LENGTH {
        Err(ValidationError::new(formatcp!("Sphere name cannot exceed {MAX_SPHERE_NAME_LENGTH} characters.")))
    } else {
        Ok(())
    }
}

/// # Returns whether a username is valid.
///
/// # Valid usernames contain only ascii alphanumeric characters, '-', '_' and have a maximum length of `MAX_USERNAME_LENGTH`
///
/// ```
/// use sharesphere_utils::checks::{check_username};
/// use sharesphere_utils::constants::MAX_USERNAME_LENGTH;
/// use sharesphere_utils::errors::AppError;
///
/// assert!(check_username("-Abc123_").is_ok());
/// assert!(check_username(" name").is_err());
/// assert!(check_username("name%").is_err());
/// assert!(check_username(&"a".repeat(MAX_USERNAME_LENGTH)).is_ok());
/// assert!(check_username(&"a".repeat(MAX_USERNAME_LENGTH + 1)).is_err());
/// ```
pub fn check_username(name: &str) -> Result<(), AppError> {
    if name.is_empty() {
        Err(AppError::new("Username cannot be empty."))
    } else if !name.chars().all(move |c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        Err(AppError::new("Username can only contain alphanumeric characters, dashes and underscores."))
    } else {
        check_string_length(name, "Username", MAX_USERNAME_LENGTH)
    }
}

pub fn validate_redirect_url(redirect_url: &str) -> Result<(), AppError> {
    let app_origin_str = get_app_origin()?;
    let app_origin = Url::parse(&app_origin_str).map_err(AppError::new)?;
    if let Ok(url) = Url::parse(redirect_url) {
        // absolute URL: check that scheme and domain correspond to app origin
        match url.origin() == app_origin.origin() {
            true => Ok(()),
            false => Err(AppError::new(format!("The redirect url {redirect_url} must have ShareSphere's origin {app_origin}."))),
        }
    } else if is_valid_pathname(redirect_url) {
        Ok(())
    } else {
        Err(AppError::new(format!("Invalid redirect url {redirect_url}: neither a valid url or pathname.")))
    }
}

fn is_valid_pathname(path: &str) -> bool {
    // Check if the path starts with '/' and is not empty
    if !path.starts_with('/') {
        return false
    }

    // Use Path to normalize and check for traversal
    let path_obj = std::path::Path::new(path);
    if path_obj.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return false
    }

    // Ensure no URL-encoded or invalid sequences
    !path.contains("//") && !path.contains("%2f") && !path.contains("%2e%2e")
}

#[cfg(test)]
mod tests {
    use sealed_test::prelude::*;
    use crate::routes::APP_ORIGIN_ENV;
    use crate::checks::{is_valid_pathname, validate_redirect_url};

    #[sealed_test]
    fn test_validate_redirect_url() {
        unsafe {
            std::env::set_var(APP_ORIGIN_ENV, "https://sharesphere.space/");
        }
        assert!(validate_redirect_url("https://sharesphere.space/valid/url").is_ok());
        assert!(validate_redirect_url("http://sharesphere.space/valid/url").is_err());
        assert!(validate_redirect_url("https://invalid.redirect/").is_err());
        assert!(validate_redirect_url("/a/path/is/ok/too").is_ok());
        assert!(validate_redirect_url("a/path/is/ok/too").is_err());
        assert!(validate_redirect_url("/a/path/is/ok/too").is_ok());
    }

    #[test]
    fn test_is_valid_pathname() {
        assert_eq!(is_valid_pathname("/valid/pathname"), true);
        assert_eq!(is_valid_pathname("invalid/pathname"), false);
        assert_eq!(is_valid_pathname("//invalid/pathname"), false);
        assert_eq!(is_valid_pathname("/invalid/../pathname"), false);
        assert_eq!(is_valid_pathname("/invalid/%2f/pathname"), false);
        assert_eq!(is_valid_pathname("/invalid/%2e%2e/pathname"), false);
    }
}