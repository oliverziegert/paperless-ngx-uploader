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

    #[tokio::test]
    async fn test_upload_file_success() {
        setup_logger();

        let mut _s = mockito::Server::new_async().await;
        let _m = _s.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(StatusCode::OK.as_u16() as usize)
            .create_async()
            .await;

        let mut cfg = Config::default();
        cfg.private_config.token = "token".to_string();
        cfg.public_config.endpoint = format!("{}/api/endpoint", _s.url());
        let client = Client::new(cfg).unwrap();

        let file_path = PathBuf::from("test_file.txt");
        fs::File::create(&file_path).unwrap();

        assert!(client.upload_file(&file_path).await.is_ok());
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

        assert!(client.upload_file(&file_path).await.is_err());
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

        let mut cfg = Config::default();
        cfg.private_config.token = "token".to_string();
        cfg.public_config.endpoint = format!("{}/api/endpoint", _s.url());
        let client = Client::new(cfg).unwrap();

        let file_path = PathBuf::from("test_file.txt");
        fs::File::create(&file_path).unwrap();

        assert!(client.upload_file(&file_path).await.is_err());
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

        let mut cfg = Config::default();
        cfg.private_config.token = "token".to_string();
        cfg.public_config.endpoint = format!("{}/api/endpoint", _s.url());
        let client = Client::new(cfg).unwrap();


        let file_path = PathBuf::from("test_file.txt");
        fs::File::create(&file_path).unwrap();

        assert!(client.upload_file(&file_path).await.is_err());
        _m.assert();

        // Delete the test file if possible
        // Ignore any errors
        let _ = fs::remove_file(file_path);
    }
}
