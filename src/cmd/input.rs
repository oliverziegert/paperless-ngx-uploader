use inquire::{Password, Text};
use log::{debug, error, info};
use std::error::Error;

/// Prompts the user to enter a Paperless-ngx endpoint URL.
///
/// Displays an interactive prompt with a placeholder example (<https://paperless.example.com>)
/// and a security warning recommending HTTPS for production use.
/// Returns the user's input as a string.
///
/// # Errors
///
/// Returns an error if the user cancels the prompt or if input reading fails.
pub fn get_endpoint_by_prompt() -> Result<String, Box<dyn Error>> {
    debug!("get_endpoint_by_prompt called");
    let input = Text::new("Endpoint")
        .with_placeholder("https://paperless.example.com")
        .with_help_message("Use HTTPS for secure connections. HTTP is only safe for localhost.")
        .prompt();
    match input {
        Ok(input) => {
            info!("Endpoint entered: {input}");
            Ok(input)
        }
        Err(e) => {
            error!("Error getting input: {e}");
            Err(e.into())
        }
    }
}

/// Prompts the user to enter a Paperless-ngx authentication token.
///
/// Displays an interactive password prompt where input is hidden for security.
/// Returns the user's input as a string.
///
/// # Errors
///
/// Returns an error if the user cancels the prompt or if input reading fails.
pub fn get_token_by_prompt() -> Result<String, Box<dyn Error>> {
    debug!("get_token_by_prompt called");
    let input = Password::new("Token").prompt();
    match input {
        Ok(input) => {
            info!("Token entered successfully");
            Ok(input)
        }
        Err(e) => {
            error!("Error getting input: {e}");
            Err(e.into())
        }
    }
}
