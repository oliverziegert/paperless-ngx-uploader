use log::debug;
use url::Url;

use super::models::CmdError;

/// Check if the URL scheme is HTTPS
///
/// Returns true if the URL uses HTTPS protocol
pub fn is_https(url: &Url) -> bool {
    let result = url.scheme() == "https";
    debug!("Called: is_https; url: {}, result: {}", url, result);
    result
}

/// Check if the URL host is localhost
///
/// Returns true if the host is localhost, 127.0.0.1, or [::1]
pub fn is_localhost(url: &Url) -> bool {
    let result = match url.host_str() {
        Some(host) => {
            host == "localhost"
                || host == "127.0.0.1"
                || host == "[::1]"
                || host == "::1"
        }
        None => false,
    };
    debug!("Called: is_localhost; url: {}, result: {}", url, result);
    result
}

/// Validate the security of an endpoint URL
///
/// An endpoint is considered secure if:
/// - It uses HTTPS protocol, OR
/// - It is a localhost connection (development/testing)
///
/// Returns Ok(()) if the endpoint is secure, or Err(InsecureConnection) if not
pub fn validate_endpoint_security(endpoint: &str) -> Result<(), CmdError> {
    debug!("Called: validate_endpoint_security; endpoint: {}", endpoint);

    let url = match Url::parse(endpoint) {
        Ok(url) => url,
        Err(e) => {
            debug!("Failed to parse URL: {}", e);
            // If we can't parse the URL, treat it as potentially insecure
            return Err(CmdError::InsecureConnection(endpoint.to_string()));
        }
    };

    // HTTPS is always secure
    if is_https(&url) {
        debug!("Endpoint is secure: uses HTTPS");
        return Ok(());
    }

    // HTTP to localhost is acceptable for development
    if is_localhost(&url) {
        debug!("Endpoint is secure: localhost connection");
        return Ok(());
    }

    // HTTP to non-localhost is insecure
    debug!("Endpoint is insecure: HTTP to non-localhost");
    Err(CmdError::InsecureConnection(endpoint.to_string()))
}
