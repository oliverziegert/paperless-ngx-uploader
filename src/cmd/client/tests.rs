use crate::cmd::config::Config;
use log::LevelFilter;
use reqwest::StatusCode;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Sets up the test logger to capture log output during tests.
fn setup_logger() {
    let _ = env_logger::builder().filter_level(LevelFilter::Debug).is_test(true).try_init();
}

#[cfg(test)]
mod http_tests {
    use super::*;
    use crate::cmd::client::helpers::{get_title_from_filename, MAX_TITLE_LENGTH};
    use crate::cmd::client::http::Client;

    #[test]
    fn test_new_client_success() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create a test config
        let mut cfg = Config::default();
        cfg.private_config.token = "test_token".to_string();
        cfg.public_config.path = config_path;

        // Create a new client
        let client = Client::new(cfg).unwrap();

        // Check that the client was created successfully
        assert_eq!(client.cfg.private_config.token, "test_token");
    }

    #[test]
    fn test_client_has_timeout_configuration() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create a test config
        let mut cfg = Config::default();
        cfg.private_config.token = "test_token".to_string();
        cfg.public_config.path = config_path;

        // Create a new client with timeout configuration
        // The client is configured with:
        // - connect_timeout: 10 seconds
        // - request_timeout: 30 seconds
        // If the timeout configuration is invalid, client creation will fail
        let client = Client::new(cfg).unwrap();

        // Verify that the client was created successfully with timeout configuration
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
    /// Returns both the TempDir (which must be kept alive) and the Config
    fn create_test_config(server_url: &str) -> (TempDir, Config) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let mut cfg = Config::default();
        cfg.private_config.token = "token".to_string();
        cfg.public_config.endpoint = format!("{}/api/endpoint", server_url);
        cfg.public_config.path = config_path;

        (temp_dir, cfg)
    }

    /// Helper function to create a test file in a temporary directory
    /// Returns both the TempDir (which must be kept alive) and the file path
    fn create_test_file(filename: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(filename);
        fs::File::create(&file_path).unwrap();
        (temp_dir, file_path)
    }

    #[tokio::test]
    async fn test_upload_file_success() {
        setup_logger();

        let mut _s = mockito::Server::new_async().await;
        let _m = _s
            .mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(StatusCode::OK.as_u16() as usize)
            .create_async()
            .await;

        let (_temp_config_dir, cfg) = create_test_config(&_s.url());
        let client = Client::new(cfg).unwrap();

        let (_temp_dir, file_path) = create_test_file("test_file.txt");
        let files = vec![file_path.clone()];

        let result = client.upload_files(&files).await;
        assert!(result.is_ok());
        let (uploaded, successful_count, failed_count) = result.unwrap();
        assert_eq!(uploaded.len(), 1);
        assert_eq!(uploaded[0], file_path);
        assert_eq!(successful_count, 1);
        assert_eq!(failed_count, 0);
        _m.assert();
    }

    #[tokio::test]
    async fn test_upload_file_error_creating_form() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let mut cfg = Config::default();
        cfg.public_config.path = config_path;
        let client = Client::new(cfg).unwrap();

        let file_path = PathBuf::from("non_existent_file.txt");
        let files = vec![file_path];

        let result = client.upload_files(&files).await;
        // When a file doesn't exist, upload_files logs the error but returns Ok with empty vec
        assert!(result.is_ok());
        let (uploaded, successful_count, failed_count) = result.unwrap();
        assert_eq!(uploaded.len(), 0);
        assert_eq!(successful_count, 0);
        assert_eq!(failed_count, 1);
    }

    #[tokio::test]
    async fn test_upload_file_error_sending_request() {
        setup_logger();

        let mut _s = mockito::Server::new_async().await;
        let _m = _s
            .mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(StatusCode::INTERNAL_SERVER_ERROR.as_u16() as usize)
            .create_async()
            .await;

        let (_temp_config_dir, cfg) = create_test_config(&_s.url());
        let client = Client::new(cfg).unwrap();

        let (_temp_dir, file_path) = create_test_file("test_file.txt");
        let files = vec![file_path.clone()];

        let result = client.upload_files(&files).await;
        // Server error is logged but upload_files returns Ok with empty vec
        assert!(result.is_ok());
        let (uploaded, successful_count, failed_count) = result.unwrap();
        assert_eq!(uploaded.len(), 0);
        assert_eq!(successful_count, 0);
        assert_eq!(failed_count, 1);
        _m.assert();
    }

    #[tokio::test]
    async fn test_upload_file_non_ok_status() {
        setup_logger();

        let mut _s = mockito::Server::new_async().await;
        let _m = _s
            .mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(StatusCode::BAD_REQUEST.as_u16() as usize)
            .create_async()
            .await;

        let (_temp_config_dir, cfg) = create_test_config(&_s.url());
        let client = Client::new(cfg).unwrap();

        let (_temp_dir, file_path) = create_test_file("test_file.txt");
        let files = vec![file_path.clone()];

        let result = client.upload_files(&files).await;
        // Non-OK status is logged but upload_files returns Ok with empty vec
        assert!(result.is_ok());
        let (uploaded, successful_count, failed_count) = result.unwrap();
        assert_eq!(uploaded.len(), 0);
        assert_eq!(successful_count, 0);
        assert_eq!(failed_count, 1);
        _m.assert();
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

        let (_temp_config_dir, cfg) = create_test_config(&server.url());
        let client = Client::new(cfg).unwrap();

        // Create test files
        let (_temp_dir1, file1) = create_test_file("test1.pdf");
        let (_temp_dir2, file2) = create_test_file("test2.pdf");
        let files = vec![file1.clone(), file2.clone()];

        // Test: Upload files in parallel
        let result = client.upload_files(&files).await;

        // Verify: All files uploaded successfully
        assert!(result.is_ok());
        let (uploaded, successful_count, failed_count) = result.unwrap();
        assert_eq!(uploaded.len(), 2);
        assert!(uploaded.contains(&file1));
        assert!(uploaded.contains(&file2));
        assert_eq!(successful_count, 2);
        assert_eq!(failed_count, 0);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_upload_files_partial_failure() {
        setup_logger();

        // Setup: Mock server that accepts first upload but rejects second
        let mut server = mockito::Server::new_async().await;

        // First request succeeds
        let mock_success = server
            .mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(200)
            .expect(1)
            .create_async()
            .await;

        // Second request fails
        let mock_fail = server
            .mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(500)
            .expect(1)
            .create_async()
            .await;

        let (_temp_config_dir, cfg) = create_test_config(&server.url());
        let client = Client::new(cfg).unwrap();

        let (_temp_dir1, file1) = create_test_file("success.pdf");
        let (_temp_dir2, file2) = create_test_file("fail.pdf");
        let files = vec![file1.clone(), file2.clone()];

        // Test: One file fails, batch continues
        let result = client.upload_files(&files).await;

        // Verify: Result is Ok (no errors thrown)
        assert!(result.is_ok());
        let (uploaded, successful_count, failed_count) = result.unwrap();

        // One file should succeed (but we can't guarantee which one due to parallel execution)
        // So we just verify that at least one succeeded and at most 2 succeeded
        assert!(uploaded.len() >= 1 && uploaded.len() <= 2);
        assert_eq!(successful_count + failed_count, 2);

        // Verify both mocks were called
        mock_success.assert_async().await;
        mock_fail.assert_async().await;
    }
}

#[cfg(test)]
mod file_ops_tests {
    use super::*;
    use crate::cmd::client::file_ops::aggregate_files;
    use regex::Regex;

    #[test]
    fn test_aggregate_files_folder_filters_matching_extension() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();

        // Create a mix of .pdf and .txt files
        fs::File::create(temp_dir.path().join("document.pdf")).unwrap();
        fs::File::create(temp_dir.path().join("notes.txt")).unwrap();
        fs::File::create(temp_dir.path().join("report.pdf")).unwrap();

        let filter = Regex::new(r"\.pdf$").unwrap();
        let result = aggregate_files(None, Some(temp_dir.path().to_path_buf()), false, &filter);

        // Only the two .pdf files should be returned
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert_eq!(paths.len(), 2);
        for path in &paths {
            assert_eq!(path.extension().unwrap().to_str().unwrap(), "pdf");
        }
    }

    #[test]
    fn test_aggregate_files_folder_non_matching_pattern() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();

        // Create files that do not match the .pdf filter
        fs::File::create(temp_dir.path().join("notes.txt")).unwrap();
        fs::File::create(temp_dir.path().join("readme.md")).unwrap();

        let filter = Regex::new(r"\.pdf$").unwrap();
        let result = aggregate_files(None, Some(temp_dir.path().to_path_buf()), false, &filter);

        // No files match the filter
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_aggregate_files_single_file_matching() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("document.pdf");
        fs::File::create(&file_path).unwrap();

        let filter = Regex::new(r"\.pdf$").unwrap();
        let result = aggregate_files(Some(file_path.clone()), None, false, &filter);

        // Single file matches the filter and is included
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], file_path);
    }

    #[test]
    fn test_aggregate_files_single_file_non_matching() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("notes.txt");
        fs::File::create(&file_path).unwrap();

        let filter = Regex::new(r"\.pdf$").unwrap();
        let result = aggregate_files(Some(file_path), None, false, &filter);

        // Single file does not match the filter and is excluded
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_aggregate_files_empty_folder() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();

        let filter = Regex::new(r"\.pdf$").unwrap();
        let result = aggregate_files(None, Some(temp_dir.path().to_path_buf()), false, &filter);

        // Empty folder yields no files
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Helper function to create a nested directory structure for testing recursive scanning
    /// Returns the TempDir (which must be kept alive)
    fn create_nested_directory_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create files in root directory
        fs::File::create(root.join("file1.pdf")).unwrap();
        fs::File::create(root.join("file2.pdf")).unwrap();
        fs::File::create(root.join("file3.txt")).unwrap();

        // Create subdirectory with files
        let subdir1 = root.join("subdir1");
        fs::create_dir(&subdir1).unwrap();
        fs::File::create(subdir1.join("file4.pdf")).unwrap();
        fs::File::create(subdir1.join("file5.txt")).unwrap();

        // Create nested subdirectory with files
        let subdir2 = subdir1.join("subdir2");
        fs::create_dir(&subdir2).unwrap();
        fs::File::create(subdir2.join("file6.pdf")).unwrap();
        fs::File::create(subdir2.join("file7.txt")).unwrap();

        temp_dir
    }

    #[test]
    fn test_aggregate_files_non_recursive_only_root() {
        setup_logger();

        let temp_dir = create_nested_directory_structure();
        let filter = Regex::new(r"\.pdf$").unwrap();

        let result = aggregate_files(None, Some(temp_dir.path().to_path_buf()), false, &filter);

        assert!(result.is_ok());
        let files = result.unwrap();
        // Should only find files in root directory, not subdirectories
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.pdf"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.pdf"));
        // Should not find files in subdirectories
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "file4.pdf"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "file6.pdf"));
    }

    #[test]
    fn test_aggregate_files_recursive_all_levels() {
        setup_logger();

        let temp_dir = create_nested_directory_structure();
        let filter = Regex::new(r"\.pdf$").unwrap();

        let result = aggregate_files(None, Some(temp_dir.path().to_path_buf()), true, &filter);

        assert!(result.is_ok());
        let files = result.unwrap();
        // Should find all PDF files in root and all subdirectories
        assert_eq!(files.len(), 4);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.pdf"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.pdf"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file4.pdf"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file6.pdf"));
        // Should not find .txt files
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "file3.txt"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "file5.txt"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "file7.txt"));
    }

    #[test]
    fn test_aggregate_files_recursive_with_different_filter() {
        setup_logger();

        let temp_dir = create_nested_directory_structure();
        let filter = Regex::new(r"\.txt$").unwrap();

        let result = aggregate_files(None, Some(temp_dir.path().to_path_buf()), true, &filter);

        assert!(result.is_ok());
        let files = result.unwrap();
        // Should find all TXT files recursively
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file3.txt"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file5.txt"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file7.txt"));
    }

    #[test]
    fn test_aggregate_files_recursive_empty_subdirectories() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create empty subdirectories
        fs::create_dir(root.join("subdir1")).unwrap();
        fs::create_dir(root.join("subdir2")).unwrap();
        fs::create_dir(root.join("subdir1").join("subdir3")).unwrap();

        let filter = Regex::new(r"\.pdf$").unwrap();

        let result = aggregate_files(None, Some(temp_dir.path().to_path_buf()), true, &filter);

        assert!(result.is_ok());
        let files = result.unwrap();
        // No files should be found in empty directories
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_aggregate_files_recursive_no_matching_files() {
        setup_logger();

        let temp_dir = create_nested_directory_structure();
        let filter = Regex::new(r"\.docx$").unwrap();

        let result = aggregate_files(None, Some(temp_dir.path().to_path_buf()), true, &filter);

        assert!(result.is_ok());
        let files = result.unwrap();
        // No .docx files should be found
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_aggregate_files_recursive_with_single_file() {
        setup_logger();

        let temp_dir = create_nested_directory_structure();

        // Create a separate temp dir for the single file to avoid duplication
        let temp_dir2 = TempDir::new().unwrap();
        let file_path = temp_dir2.path().join("extra.pdf");
        fs::File::create(&file_path).unwrap();

        let filter = Regex::new(r"\.pdf$").unwrap();

        let result = aggregate_files(
            Some(file_path.clone()),
            Some(temp_dir.path().to_path_buf()),
            true,
            &filter,
        );

        assert!(result.is_ok());
        let files = result.unwrap();
        // Should find 4 PDFs recursively + 1 specific file
        assert_eq!(files.len(), 5);
        assert!(files.contains(&file_path));
    }

    #[test]
    fn test_aggregate_files_recursive_only_deep_nested_files() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create empty root and first level subdirectory
        let subdir1 = root.join("subdir1");
        fs::create_dir(&subdir1).unwrap();

        // Create files only in deeply nested directory
        let subdir2 = subdir1.join("subdir2");
        fs::create_dir(&subdir2).unwrap();
        fs::File::create(subdir2.join("deep1.pdf")).unwrap();
        fs::File::create(subdir2.join("deep2.pdf")).unwrap();

        let filter = Regex::new(r"\.pdf$").unwrap();

        let result = aggregate_files(None, Some(temp_dir.path().to_path_buf()), true, &filter);

        assert!(result.is_ok());
        let files = result.unwrap();
        // Should find files in deeply nested directory
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "deep1.pdf"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "deep2.pdf"));
    }

    #[test]
    fn test_aggregate_files_recursive_vs_non_recursive_comparison() {
        setup_logger();

        let temp_dir = create_nested_directory_structure();
        let filter = Regex::new(r"\.pdf$").unwrap();

        // Non-recursive scan
        let non_recursive_result = aggregate_files(
            None,
            Some(temp_dir.path().to_path_buf()),
            false,
            &filter,
        );
        assert!(non_recursive_result.is_ok());
        let non_recursive_files = non_recursive_result.unwrap();

        // Recursive scan
        let recursive_result = aggregate_files(
            None,
            Some(temp_dir.path().to_path_buf()),
            true,
            &filter,
        );
        assert!(recursive_result.is_ok());
        let recursive_files = recursive_result.unwrap();

        // Recursive should find more files than non-recursive
        assert!(recursive_files.len() > non_recursive_files.len());
        assert_eq!(non_recursive_files.len(), 2); // Only root level
        assert_eq!(recursive_files.len(), 4); // Root + subdirectories

        // All non-recursive files should be included in recursive results
        for file in &non_recursive_files {
            assert!(recursive_files.contains(file));
        }
    }
}
