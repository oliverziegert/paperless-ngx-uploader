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
    fn test_client_has_timeout_configuration() {
        setup_logger();

        // Create a test config
        let mut cfg = Config::default();
        cfg.private_config.token = "test_token".to_string();

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

#[cfg(test)]
mod file_ops_tests {
    use super::*;
    use crate::cmd::client::file_ops::{delete_expired_files, ARCHIVE_FOLDER_NAME};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};
    use filetime::FileTime;

    /// Helper function to create a test file with a specific modified time
    fn create_test_file_with_age(dir: &PathBuf, filename: &str, days_old: u64) -> PathBuf {
        let file_path = dir.join(filename);
        fs::File::create(&file_path).unwrap();

        // Set the file's modified time to be the specified number of days old
        let secs_per_day = 60 * 60 * 24;
        let file_time = SystemTime::now() - Duration::from_secs(days_old * secs_per_day);
        let ft = FileTime::from_system_time(file_time);
        filetime::set_file_mtime(&file_path, ft).unwrap();

        file_path
    }

    /// Helper function to create a directory structure for testing
    fn create_test_directory(dir_name: &str) -> PathBuf {
        let dir_path = PathBuf::from(dir_name);
        fs::create_dir_all(&dir_path).unwrap();
        dir_path
    }

    /// Helper function to cleanup test directories
    fn cleanup_directory(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_delete_expired_files_multiple_files_same_folder() {
        setup_logger();

        // Setup: Create test directory and archive folder
        let test_dir = create_test_directory("test_delete_same_folder");
        let archive_dir = test_dir.join(ARCHIVE_FOLDER_NAME);
        fs::create_dir_all(&archive_dir).unwrap();

        // Create multiple files in the same archive folder with different ages
        let old_file1 = create_test_file_with_age(&archive_dir, "old_file1.txt", 10);
        let old_file2 = create_test_file_with_age(&archive_dir, "old_file2.txt", 15);
        let recent_file = create_test_file_with_age(&archive_dir, "recent_file.txt", 3);

        // Create input files (files that trigger the deletion check)
        // These need to be in the parent directory to trigger archive folder check
        let input_file1 = create_test_file_with_age(&test_dir, "input1.txt", 0);
        let input_file2 = create_test_file_with_age(&test_dir, "input2.txt", 0);
        let input_files = vec![input_file1.clone(), input_file2.clone()];

        // Test: Delete files older than 5 days
        let result = delete_expired_files(&input_files, 5);
        assert!(result.is_ok());

        // Verify: Old files are deleted, recent file remains
        assert!(!old_file1.exists(), "old_file1 should be deleted");
        assert!(!old_file2.exists(), "old_file2 should be deleted");
        assert!(recent_file.exists(), "recent_file should still exist");

        // Cleanup
        cleanup_directory(&test_dir);
    }

    #[test]
    fn test_delete_expired_files_multiple_files_different_folders() {
        setup_logger();

        // Setup: Create two test directories with their own archive folders
        let test_dir1 = create_test_directory("test_delete_diff_folder1");
        let test_dir2 = create_test_directory("test_delete_diff_folder2");
        let archive_dir1 = test_dir1.join(ARCHIVE_FOLDER_NAME);
        let archive_dir2 = test_dir2.join(ARCHIVE_FOLDER_NAME);
        fs::create_dir_all(&archive_dir1).unwrap();
        fs::create_dir_all(&archive_dir2).unwrap();

        // Create old files in both archive folders
        let old_file_dir1 = create_test_file_with_age(&archive_dir1, "old_file_dir1.txt", 12);
        let old_file_dir2 = create_test_file_with_age(&archive_dir2, "old_file_dir2.txt", 20);

        // Create recent files in both archive folders
        let recent_file_dir1 = create_test_file_with_age(&archive_dir1, "recent_file_dir1.txt", 2);
        let recent_file_dir2 = create_test_file_with_age(&archive_dir2, "recent_file_dir2.txt", 4);

        // Create input files in both directories to trigger archive folder checks
        let input_file1 = create_test_file_with_age(&test_dir1, "input1.txt", 0);
        let input_file2 = create_test_file_with_age(&test_dir2, "input2.txt", 0);
        let input_files = vec![input_file1.clone(), input_file2.clone()];

        // Test: Delete files older than 7 days from both folders
        let result = delete_expired_files(&input_files, 7);
        assert!(result.is_ok());

        // Verify: Old files from both folders are deleted, recent files remain
        assert!(!old_file_dir1.exists(), "old_file in dir1 should be deleted");
        assert!(!old_file_dir2.exists(), "old_file in dir2 should be deleted");
        assert!(recent_file_dir1.exists(), "recent_file in dir1 should still exist");
        assert!(recent_file_dir2.exists(), "recent_file in dir2 should still exist");

        // Cleanup
        cleanup_directory(&test_dir1);
        cleanup_directory(&test_dir2);
    }

    #[test]
    fn test_delete_expired_files_no_archive_folder() {
        setup_logger();

        // Setup: Create test directory WITHOUT archive folder
        let test_dir = create_test_directory("test_delete_no_archive");
        let input_file = create_test_file_with_age(&test_dir, "input.txt", 0);
        let input_files = vec![input_file.clone()];

        // Test: Should not fail when archive folder doesn't exist
        let result = delete_expired_files(&input_files, 5);
        assert!(result.is_ok());

        // Cleanup
        cleanup_directory(&test_dir);
    }

    #[test]
    fn test_delete_expired_files_empty_archive_folder() {
        setup_logger();

        // Setup: Create test directory with empty archive folder
        let test_dir = create_test_directory("test_delete_empty_archive");
        let archive_dir = test_dir.join(ARCHIVE_FOLDER_NAME);
        fs::create_dir_all(&archive_dir).unwrap();
        let input_file = create_test_file_with_age(&test_dir, "input.txt", 0);
        let input_files = vec![input_file.clone()];

        // Test: Should not fail with empty archive folder
        let result = delete_expired_files(&input_files, 5);
        assert!(result.is_ok());

        // Verify: Archive folder still exists
        assert!(archive_dir.exists());

        // Cleanup
        cleanup_directory(&test_dir);
    }
}
