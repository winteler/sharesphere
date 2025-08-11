use crate::errors::AppError;

pub fn check_string_length(
    input: &str,
    input_name: &str,
    max_length: usize,
) -> Result<(), AppError> {
    match input.len() {
        x if x > max_length => Err(AppError::new(format!("{input_name} exceeds the maximum length {max_length}."))),
        _ => Ok(()),
    }
}