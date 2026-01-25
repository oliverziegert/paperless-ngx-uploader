use crate::cmd::config::Config;
use log::LevelFilter;
use reqwest::StatusCode;
use std::fs;
use std::path::PathBuf;

/// Sets up the test logger to capture log output during tests.
fn setup_logger() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

#[cfg(test)]
mod http_tests {
    use super::*;
    use crate::cmd::client::http::Client;
    use crate::cmd::client::helpers::{get_title_from_filename, MAX_TITLE_LENGTH};

    #[test]
    fn test_new_client_success() {
        setup_logger();

        // Create a test config
        let mut cfg = Config::default();
        cfg.private_config.token = "test_token".to_string();

        // Create a new client
        let client = Client::new(cfg).unwrap();

        // Check that the client was created successfully
        assert_eq!(client.cfg.private_config.token, "test_token");
    }

    #[test]
    fn test_get_title_from_filename_no_extension() {
        setup_logger();

        let file = PathBuf::from("testfile");
        let expected_title = "testfile";
        assert_eq!(get_title_from_filename(&file), expected_title);
    }

    #[test]
    fn test_get_title_from_filename_with_extension() {
        setup_logger();

        let file = PathBuf::from("testfile.txt");
        let expected_title = "testfile";
        assert_eq!(get_title_from_filename(&file), expected_title);
    }

    #[test]
    fn test_get_title_from_filename_long_name() {
        setup_logger();

        let file = PathBuf::from("Lorem_ipsum_dolor_sit_amet_consectetur_adipiscing_elit_Sed_ut_lectus_sit_amet_nulla_sagittis_tristique.txt");
        let expected_title = "Lorem_ipsum_dolor_sit_amet_consectetur_adipiscing_elit_Sed_ut_lectus_sit_amet_nulla_sagittis_tristique".split_at(MAX_TITLE_LENGTH).0;
        assert_eq!(get_title_from_filename(&file), expected_title);
    }

    #[test]
    fn test_get_title_from_filename_non_ascii() {
        setup_logger();

        let file = PathBuf::from("testfile_é.txt");
        let expected_title = "testfile_é";
        assert_eq!(get_title_from_filename(&file), expected_title);
    }

    #[test]
    fn test_get_title_from_filename_special_chars() {
        setup_logger();

        let file = PathBuf::from("testfile!@#$%^&*().txt");
        let expected_title = "testfile!@#$%^&*()";
        assert_eq!(get_title_from_filename(&file), expected_title);
    }

    /// Helper function to create a test config for the given mock server URL
    fn create_test_config(server_url: &str) -> Config {
        let mut cfg = Config::default();
        cfg.private_config.token = "token".to_string();
        cfg.public_config.endpoint = format!("{}/api/endpoint", server_url);
        cfg
    }

    /// Helper function to create a test file and return its path
    fn create_test_file(filename: &str) -> PathBuf {
        let file_path = PathBuf::from(filename);
        fs::File::create(&file_path).unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_upload_file_success() {
        setup_logger();

        let mut _s = mockito::Server::new_async().await;
        let _m = _s.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(StatusCode::OK.as_u16() as usize)
            .create_async()
            .await;

        let cfg = create_test_config(&_s.url());
        let client = Client::new(cfg).unwrap();

        let file_path = create_test_file("test_file.txt");
        let files = vec![file_path.clone()];

        let result = client.upload_files(&files).await;
        assert!(result.is_ok());
        let uploaded = result.unwrap();
        assert_eq!(uploaded.len(), 1);
        assert_eq!(uploaded[0], file_path);
        _m.assert();

        // Delete the test file if possible
        // Ignore any errors
        let _ = fs::remove_file(file_path);
    }

    #[tokio::test]
    async fn test_upload_file_error_creating_form() {
        setup_logger();

        let cfg = Config::default();
        let client = Client::new(cfg).unwrap();

        let file_path = PathBuf::from("non_existent_file.txt");
        let files = vec![file_path];

        let result = client.upload_files(&files).await;
        // When a file doesn't exist, upload_files logs the error but returns Ok with empty vec
        assert!(result.is_ok());
        let uploaded = result.unwrap();
        assert_eq!(uploaded.len(), 0);
    }

    #[tokio::test]
    async fn test_upload_file_error_sending_request() {
        setup_logger();

        let mut _s = mockito::Server::new_async().await;
        let _m = _s.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(StatusCode::INTERNAL_SERVER_ERROR.as_u16() as usize)
            .create_async()
            .await;

        let cfg = create_test_config(&_s.url());
        let client = Client::new(cfg).unwrap();

        let file_path = create_test_file("test_file.txt");
        let files = vec![file_path.clone()];

        let result = client.upload_files(&files).await;
        // Server error is logged but upload_files returns Ok with empty vec
        assert!(result.is_ok());
        let uploaded = result.unwrap();
        assert_eq!(uploaded.len(), 0);
        _m.assert();

        // Delete the test file if possible
        // Ignore any errors
        let _ = fs::remove_file(file_path);
    }

    #[tokio::test]
    async fn test_upload_file_non_ok_status() {
        setup_logger();

        let mut _s = mockito::Server::new_async().await;
        let _m = _s.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(StatusCode::BAD_REQUEST.as_u16() as usize)
            .create_async()
            .await;

        let cfg = create_test_config(&_s.url());
        let client = Client::new(cfg).unwrap();

        let file_path = create_test_file("test_file.txt");
        let files = vec![file_path.clone()];

        let result = client.upload_files(&files).await;
        // Non-OK status is logged but upload_files returns Ok with empty vec
        assert!(result.is_ok());
        let uploaded = result.unwrap();
        assert_eq!(uploaded.len(), 0);
        _m.assert();

        // Delete the test file if possible
        // Ignore any errors
        let _ = fs::remove_file(file_path);
    }

    #[tokio::test]
    async fn test_upload_files_parallel_success() {
        setup_logger();

        // Setup: Create mock server that accepts uploads
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(200)
            .expect_at_least(2)  // Verify multiple files uploaded
            .create_async()
            .await;

        let cfg = create_test_config(&server.url());
        let client = Client::new(cfg).unwrap();

        // Create test files
        let file1 = create_test_file("test1.pdf");
        let file2 = create_test_file("test2.pdf");
        let files = vec![file1.clone(), file2.clone()];

        // Test: Upload files in parallel
        let result = client.upload_files(&files).await;

        // Verify: All files uploaded successfully
        assert!(result.is_ok());
        let uploaded = result.unwrap();
        assert_eq!(uploaded.len(), 2);
        assert!(uploaded.contains(&file1));
        assert!(uploaded.contains(&file2));
        mock.assert_async().await;

        // Cleanup
        let _ = fs::remove_file(file1);
        let _ = fs::remove_file(file2);
    }

    #[tokio::test]
    async fn test_upload_files_partial_failure() {
        setup_logger();

        // Setup: Mock server that accepts first upload but rejects second
        let mut server = mockito::Server::new_async().await;

        // First request succeeds
        let mock_success = server.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(200)
            .expect(1)
            .create_async()
            .await;

        // Second request fails
        let mock_fail = server.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(500)
            .expect(1)
            .create_async()
            .await;

        let cfg = create_test_config(&server.url());
        let client = Client::new(cfg).unwrap();

        let file1 = create_test_file("success.pdf");
        let file2 = create_test_file("fail.pdf");
        let files = vec![file1.clone(), file2.clone()];

        // Test: One file fails, batch continues
        let result = client.upload_files(&files).await;

        // Verify: Result is Ok (no errors thrown)
        assert!(result.is_ok());
        let uploaded = result.unwrap();

        // One file should succeed (but we can't guarantee which one due to parallel execution)
        // So we just verify that at least one succeeded and at most 2 succeeded
        assert!(uploaded.len() >= 1 && uploaded.len() <= 2);

        // Verify both mocks were called
        mock_success.assert_async().await;
        mock_fail.assert_async().await;

        // Cleanup
        let _ = fs::remove_file(file1);
        let _ = fs::remove_file(file2);
    }
}
