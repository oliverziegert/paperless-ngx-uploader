use log::{debug, error, info};
use std::error::Error;
use std::io::{self, Write};

use super::models::CmdError;
use super::url_validator::{validate_endpoint_format, validate_endpoint_security};

const ENDPOINT_PLACEHOLDER: &str = "https://paperless.example.com";

/// Prompts the user to enter a Paperless-ngx endpoint URL.
///
/// Prints a security help message and displays a prompt with a placeholder
/// example (`https://paperless.example.com`). If the user presses Enter
/// without typing anything, the placeholder value is used as the default.
///
/// Validates the endpoint URL format and security, re-prompting the user
/// if validation fails. Continues until a valid endpoint is entered.
///
/// Returns the user's input (or the placeholder) as a string.
///
/// # Errors
///
/// Returns an error if reading from stdin fails or if stdout cannot be flushed.
pub fn get_endpoint_by_prompt() -> Result<String, Box<dyn Error>> {
    debug!("get_endpoint_by_prompt called");

    println!("Use HTTPS for secure connections. HTTP is only safe for localhost.");

    loop {
        print!("Endpoint [{ENDPOINT_PLACEHOLDER}]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let endpoint = input.trim().to_string();

        let endpoint = if endpoint.is_empty() {
            debug!("Empty input; using placeholder default");
            ENDPOINT_PLACEHOLDER.to_string()
        } else {
            endpoint
        };

        debug!("Validating endpoint: {endpoint}");

        // Validate endpoint format
        if let Err(e) = validate_endpoint_format(&endpoint) {
            if let CmdError::InvalidUrl(msg) = e {
                error!("Invalid endpoint format: {msg}");
                eprintln!("Error: {msg}");
                eprintln!("Please try again.\n");
                continue;
            }
            error!("Unexpected error during format validation: {e}");
            return Err(Box::new(e));
        }

        // Validate endpoint security
        if let Err(e) = validate_endpoint_security(&endpoint) {
            if let CmdError::InsecureConnection(url) = e {
                error!("Insecure connection to: {url}");
                eprintln!("Error: Insecure HTTP connection to {url}.");
                eprintln!("Use HTTPS for secure connections, or use localhost for development.");
                eprintln!("Please try again.\n");
                continue;
            }
            error!("Unexpected error during security validation: {e}");
            return Err(Box::new(e));
        }

        info!("Endpoint entered and validated: {endpoint}");
        return Ok(endpoint);
    }
}

/// Prompts the user to enter a Paperless-ngx authentication token.
///
/// Displays a password prompt where input is hidden for security using
/// [`rpassword`]. Returns the user's input as a string.
///
/// # Errors
///
/// Returns an error if reading the password from the terminal fails.
pub fn get_token_by_prompt() -> Result<String, Box<dyn Error>> {
    debug!("get_token_by_prompt called");

    println!("Find your token in Paperless-ngx Settings > API. Input will be hidden.");

    let token = rpassword::prompt_password("Token: ").map_err(|e| {
        error!("Error getting input: {e}");
        e
    })?;

    info!("Token entered successfully");
    Ok(token)
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
