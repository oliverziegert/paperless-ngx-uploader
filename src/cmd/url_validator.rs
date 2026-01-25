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

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    /// Sets up the test logger to capture log output during tests.
    fn setup_logger() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[cfg(test)]
    mod is_https_tests {
        use super::*;

        #[test]
        fn test_https_url_returns_true() {
            setup_logger();

            let url = Url::parse("https://example.com").unwrap();
            assert!(is_https(&url));
        }

        #[test]
        fn test_https_url_with_port_returns_true() {
            setup_logger();

            let url = Url::parse("https://example.com:8443").unwrap();
            assert!(is_https(&url));
        }

        #[test]
        fn test_https_url_with_path_returns_true() {
            setup_logger();

            let url = Url::parse("https://example.com/api/documents").unwrap();
            assert!(is_https(&url));
        }

        #[test]
        fn test_http_url_returns_false() {
            setup_logger();

            let url = Url::parse("http://example.com").unwrap();
            assert!(!is_https(&url));
        }

        #[test]
        fn test_http_url_with_port_returns_false() {
            setup_logger();

            let url = Url::parse("http://example.com:8080").unwrap();
            assert!(!is_https(&url));
        }
    }

    #[cfg(test)]
    mod is_localhost_tests {
        use super::*;

        #[test]
        fn test_localhost_returns_true() {
            setup_logger();

            let url = Url::parse("http://localhost").unwrap();
            assert!(is_localhost(&url));
        }

        #[test]
        fn test_localhost_with_port_returns_true() {
            setup_logger();

            let url = Url::parse("http://localhost:8000").unwrap();
            assert!(is_localhost(&url));
        }

        #[test]
        fn test_127_0_0_1_returns_true() {
            setup_logger();

            let url = Url::parse("http://127.0.0.1").unwrap();
            assert!(is_localhost(&url));
        }

        #[test]
        fn test_127_0_0_1_with_port_returns_true() {
            setup_logger();

            let url = Url::parse("http://127.0.0.1:8000").unwrap();
            assert!(is_localhost(&url));
        }

        #[test]
        fn test_ipv6_localhost_bracketed_returns_true() {
            setup_logger();

            let url = Url::parse("http://[::1]").unwrap();
            assert!(is_localhost(&url));
        }

        #[test]
        fn test_ipv6_localhost_with_port_returns_true() {
            setup_logger();

            let url = Url::parse("http://[::1]:8000").unwrap();
            assert!(is_localhost(&url));
        }

        #[test]
        fn test_external_host_returns_false() {
            setup_logger();

            let url = Url::parse("http://example.com").unwrap();
            assert!(!is_localhost(&url));
        }

        #[test]
        fn test_external_ip_returns_false() {
            setup_logger();

            let url = Url::parse("http://192.168.1.1").unwrap();
            assert!(!is_localhost(&url));
        }

        #[test]
        fn test_external_ip_with_port_returns_false() {
            setup_logger();

            let url = Url::parse("http://10.0.0.1:8080").unwrap();
            assert!(!is_localhost(&url));
        }
    }

    #[cfg(test)]
    mod validate_endpoint_security_tests {
        use super::*;

        #[test]
        fn test_https_external_host_is_secure() {
            setup_logger();

            let result = validate_endpoint_security("https://example.com");
            assert!(result.is_ok());
        }

        #[test]
        fn test_https_with_port_is_secure() {
            setup_logger();

            let result = validate_endpoint_security("https://example.com:8443");
            assert!(result.is_ok());
        }

        #[test]
        fn test_https_with_path_is_secure() {
            setup_logger();

            let result = validate_endpoint_security("https://paperless.example.com/api/documents/post_document/");
            assert!(result.is_ok());
        }

        #[test]
        fn test_http_localhost_is_secure() {
            setup_logger();

            let result = validate_endpoint_security("http://localhost");
            assert!(result.is_ok());
        }

        #[test]
        fn test_http_localhost_with_port_is_secure() {
            setup_logger();

            let result = validate_endpoint_security("http://localhost:8000");
            assert!(result.is_ok());
        }

        #[test]
        fn test_http_127_0_0_1_is_secure() {
            setup_logger();

            let result = validate_endpoint_security("http://127.0.0.1");
            assert!(result.is_ok());
        }

        #[test]
        fn test_http_127_0_0_1_with_port_is_secure() {
            setup_logger();

            let result = validate_endpoint_security("http://127.0.0.1:8000");
            assert!(result.is_ok());
        }

        #[test]
        fn test_http_ipv6_localhost_is_secure() {
            setup_logger();

            let result = validate_endpoint_security("http://[::1]");
            assert!(result.is_ok());
        }

        #[test]
        fn test_http_ipv6_localhost_with_port_is_secure() {
            setup_logger();

            let result = validate_endpoint_security("http://[::1]:8000");
            assert!(result.is_ok());
        }

        #[test]
        fn test_http_external_host_is_insecure() {
            setup_logger();

            let result = validate_endpoint_security("http://example.com");
            assert!(result.is_err());

            // Verify it's the correct error type
            match result {
                Err(CmdError::InsecureConnection(url)) => {
                    assert_eq!(url, "http://example.com");
                }
                _ => panic!("Expected InsecureConnection error"),
            }
        }

        #[test]
        fn test_http_external_ip_is_insecure() {
            setup_logger();

            let result = validate_endpoint_security("http://192.168.1.1");
            assert!(result.is_err());

            // Verify it's the correct error type
            match result {
                Err(CmdError::InsecureConnection(url)) => {
                    assert_eq!(url, "http://192.168.1.1");
                }
                _ => panic!("Expected InsecureConnection error"),
            }
        }

        #[test]
        fn test_http_external_host_with_port_is_insecure() {
            setup_logger();

            let result = validate_endpoint_security("http://paperless.example.com:8000");
            assert!(result.is_err());

            // Verify it's the correct error type
            match result {
                Err(CmdError::InsecureConnection(url)) => {
                    assert_eq!(url, "http://paperless.example.com:8000");
                }
                _ => panic!("Expected InsecureConnection error"),
            }
        }

        #[test]
        fn test_http_external_host_with_path_is_insecure() {
            setup_logger();

            let result = validate_endpoint_security("http://paperless.example.com/api/documents/");
            assert!(result.is_err());

            // Verify it's the correct error type
            match result {
                Err(CmdError::InsecureConnection(url)) => {
                    assert_eq!(url, "http://paperless.example.com/api/documents/");
                }
                _ => panic!("Expected InsecureConnection error"),
            }
        }

        #[test]
        fn test_invalid_url_returns_error() {
            setup_logger();

            let result = validate_endpoint_security("not-a-valid-url");
            assert!(result.is_err());

            // Invalid URLs are treated as insecure
            match result {
                Err(CmdError::InsecureConnection(url)) => {
                    assert_eq!(url, "not-a-valid-url");
                }
                _ => panic!("Expected InsecureConnection error for invalid URL"),
            }
        }

        #[test]
        fn test_empty_string_returns_error() {
            setup_logger();

            let result = validate_endpoint_security("");
            assert!(result.is_err());

            // Empty string is treated as insecure
            match result {
                Err(CmdError::InsecureConnection(url)) => {
                    assert_eq!(url, "");
                }
                _ => panic!("Expected InsecureConnection error for empty URL"),
            }
        }

        #[test]
        fn test_https_localhost_is_secure() {
            setup_logger();

            // HTTPS to localhost should also be secure
            let result = validate_endpoint_security("https://localhost:8443");
            assert!(result.is_ok());
        }
    }
}
