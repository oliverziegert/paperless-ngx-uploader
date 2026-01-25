use crate::cmd::config::Config;
use crate::cmd::models::CmdError;
use http::header;
use log::{debug, error, info};
use regex::Regex;
use reqwest::blocking::multipart;
use std::cmp::min;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

const HEADER_AUTH_PREFIX: &str = "Token ";
const MAX_TITLE_LENGTH: usize = 64;
const ARCHIVE_FOLDER_NAME: &str = "archive";
const SECS_PER_DAY: u64 = 60 * 60 * 24;


pub(crate) struct Client {
    cfg: Config,
    http: reqwest::blocking::Client,
}

impl Client {
    /// Creates a new `Client` with the given `Config`.
    ///
    /// The `Config` is used to create a `reqwest::blocking::Client` with the
    /// `Authorization` header set to the token configured in the `Config`.
    ///
    /// # Errors
    ///
    /// Returns `Err(CmdError::ClientCreationFailed)` if the HTTP client cannot be created.
    pub(crate) fn new(cfg: Config) -> Result<Self, CmdError> {
        debug!("Creating new Client with provided Config");

        let mut header = header::HeaderMap::new();
        debug!("HeaderMap created");

        let header_auth_value = format!("{}{}", HEADER_AUTH_PREFIX, cfg.private_config.token);
        let mut header_auth_value = header::HeaderValue::from_str(header_auth_value.as_str())
            .map_err(|_| CmdError::ClientCreationFailed)?;
        header_auth_value.set_sensitive(true);
        header.insert(header::AUTHORIZATION, header_auth_value);
        debug!("Authorization header set");

        header.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        debug!("Accept header set");

        let client = reqwest::blocking::Client::builder()
            .default_headers(header)
            .build()
            .map_err(|_| CmdError::ClientCreationFailed)?;
        debug!("HTTP Client created successfully");

        Ok(Self { cfg, http: client })
    }

    /// Uploads documents to Paperless-ngx with optional archival and cleanup.
    ///
    /// This method aggregates files from either a single file or a folder, filters
    /// them by the provided regex pattern, uploads them to the Paperless-ngx endpoint,
    /// and optionally archives uploaded files and deletes expired files.
    ///
    /// # Arguments
    ///
    /// * `file` - Optional path to a single file to upload
    /// * `folder` - Optional path to a folder containing files to upload
    /// * `filter` - Regex pattern to filter file names
    /// * `archive` - If true, successfully uploaded files are archived
    /// * `period` - Number of days after which files are considered expired (used with `delete`)
    /// * `delete` - If true, files older than `period` days are deleted
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File aggregation fails (invalid regex, folder not readable)
    /// - File upload fails (network error, authentication failure, server error)
    /// - Archival fails (unable to create archive folder, file move error)
    /// - Deletion fails (unable to delete expired files)
    pub(crate) fn upload(
        &self,
        file: Option<PathBuf>,
        folder: Option<PathBuf>,
        filter: String,
        archive: bool,
        period: usize,
        delete: bool,
    ) -> Result<(), Box<dyn Error>> {
        debug!("Called: Client::upload");
        let files = &aggregate_files(file, folder, filter)?;

        if files.is_empty() {
            info!("No files found. Nothing to do.");
            return Ok(());
        }

        let files_to_archive = match self.upload_files(files) {
            Ok(files) => files,
            Err(e) => {
                error!("Error uploading files: {}", e);
                return Err(e);
            }
        };

        if archive {
            archive_files(&files_to_archive)?;
        }

        if delete {
            delete_expired_files(files, period)?;
        }

        Ok(())
    }

    /// Uploads a list of files to Paperless-ngx.
    ///
    /// This method iterates through the provided files and attempts to upload
    /// each one. Successfully uploaded files are collected and returned. If an
    /// individual file upload fails, the error is logged and the method continues
    /// with the remaining files.
    ///
    /// # Arguments
    ///
    /// * `files` - A vector of file paths to upload
    ///
    /// # Returns
    ///
    /// Returns a vector of paths for files that were successfully uploaded.
    ///
    /// # Errors
    ///
    /// This method does not return errors. Individual file upload failures are
    /// logged but do not stop the upload process.
    fn upload_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        debug!("Called: Client::upload_files");
        let mut files_archived: Vec<PathBuf> = Vec::new();
        for file in files.iter() {
            match self.upload_file(file) {
                Ok(_) => {
                    debug!("File uploaded successfully");
                    files_archived.push(file.clone())
                }
                Err(e) => {
                    error!("Error uploading file: {}", e);
                }
            }
        }
        Ok(files_archived)
    }

    /// Uploads a single file to Paperless-ngx.
    ///
    /// This method creates a multipart form containing the file and its title
    /// (derived from the filename), then posts it to the configured Paperless-ngx
    /// endpoint. The upload is considered successful only if the server responds
    /// with HTTP 200 OK.
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the file to upload
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The multipart form cannot be created
    /// - The HTTP request fails (network error, authentication failure)
    /// - The server returns a non-200 status code
    fn upload_file(&self, file: &PathBuf) -> Result<(), Box<dyn Error>> {
        debug!("Called: Client::upload_file");

        let mut form = match multipart::Form::new().file("document", file) {
            Ok(form) => form,
            Err(e) => {
                error!("Error creating multipart form: {}", e);
                return Err(e.into());
            }
        };

        let title = get_title_from_filename(file);
        form = form.text("title", title.clone());
        debug!("file_name: {}", &title);

        let response = match self.http.post(&self.cfg.public_config.endpoint).multipart(form).send() {
            Ok(response) => response,
            Err(e) => {
                error!("Error uploading file: {}", e);
                return Err(e.into());
            }
        };

        match response.status() {
            reqwest::StatusCode::OK => {
                info!("File {} uploaded successfully", &title);
                Ok(())
            }
            _ => {
                error!("Error uploading file: {}", response.status());
                Err(format!("Error uploading file: {}", response.status()).into())
            }
        }
    }
}

/// Combines file and folder paths into a single vector.
///
/// - If the `file` argument is provided, it adds the file to the vector.
/// - If the `folder` argument is provided, it reads all entries in the folder
///   and adds them to the vector.
/// - The `filter` argument is a regex pattern used to filter file names.
///
/// Returns a vector of `PathBuf` on success, or an error if the folder cannot be read.
fn aggregate_files(
    file: Option<PathBuf>,
    folder: Option<PathBuf>,
    filter: String,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths = Vec::new();
    let regex = Regex::new(&filter)?;

    if let Some(folder_path) = folder {
        let entries = fs::read_dir(folder_path)?;
        for entry in entries {
            let entry_path = entry?.path();
            if let Some(file_name) = entry_path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if regex.is_match(file_name_str) {
                        paths.push(entry_path);
                    }
                }
            }
        }
    }

    if let Some(file_path) = file {
        if let Some(file_name) = file_path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                if regex.is_match(file_name_str) {
                    paths.push(file_path);
                }
            }
        }
    }

    Ok(paths)
}

/// Archives a list of files based on the provided period.
///
/// This function iterates over the provided files and archives each one
/// if its age exceeds the specified period in days.
///
/// # Arguments
///
/// * `files` - A reference to a vector of `PathBuf` representing the files to be archived.
/// * `period` - The period in days beyond which files should be archived.
///
/// # Errors
///
/// Returns an error if any file operation fails during the archiving process.
fn archive_files(files: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    debug!("Starting archive_files");
    for file in files {
        debug!("Attempting to archive file: {:?}", file);
        match archive_file(file) {
            Ok(_) => debug!("File archived successfully: {:?}", file),
            Err(e) => error!("Error archiving file {:?}: {}", file, e),
        }
    }
    debug!("Completed archive_files");
    Ok(())
}


/// Archives a single file if it is older than the specified period in days.
///
/// The function first checks if the file's creation date is in the future,
/// and if so, skips the file. Otherwise, it calculates the number of days
/// since the file was created and checks if it exceeds the specified period.
/// If it does, the file is moved to the "archive" folder in the same parent
/// directory.
///
/// # Arguments
///
/// * `file` - The file to be archived.
/// * `period` - The period in days beyond which files should be archived.
///
/// # Errors
///
/// Returns an error if any file operation fails during the archiving process.
fn archive_file(file: &PathBuf) -> Result<(), Box<dyn Error>> {
    debug!("Called: Client::archive_file");
    let parent = file.parent()
        .ok_or_else(|| CmdError::InvalidFilePath(file.display().to_string()))?;
    let archive_folder = parent.join(ARCHIVE_FOLDER_NAME);
    debug!("Creating archive folder: {}", archive_folder.display());
    fs::create_dir_all(&archive_folder)?;
    let file_name = file.file_name()
        .ok_or_else(|| CmdError::InvalidFilePath(file.display().to_string()))?;
    let target_path = archive_folder.join(file_name);
    debug!("Moving file {} to {}", file.display(), target_path.display());

    // Move the file to the archive folder
    fs::rename(file, &target_path)?;
    info!("File {} moved to archive folder", file.display());
    Ok(())
}


/// Deletes files that are older than a specified period.
///
/// Iterates over a list of files and attempts to delete each one
/// that is older than the specified period in days.
///
/// # Arguments
///
/// * `files` - A reference to a vector of `PathBuf` representing the files to be checked.
/// * `period` - The period in days beyond which files should be deleted.
///
/// # Errors
///
/// Logs an error if any file cannot be deleted.
fn delete_expired_files(files: &[PathBuf], period: usize) -> Result<(), Box<dyn Error>> {
    debug!("Called: Client::delete_expired_files");
    // Iterate over each file in the list
    for file in files.iter() {
        debug!("Attempting to delete archived files for: {}", file.display());
        let parent = file.parent()
            .ok_or_else(|| CmdError::InvalidFilePath(file.display().to_string()))?;
        let archive_folder = parent.join(ARCHIVE_FOLDER_NAME);
        if !archive_folder.is_dir() {
            debug!("Archive folder does not exist: {}", archive_folder.display());
            continue;
        }
        let archived_files = fs::read_dir(&archive_folder)?.collect::<Vec<_>>();
        for archived_file in archived_files {
            let archived_file = archived_file?.path();
            match delete_expired_file(&archived_file, period) {
                Ok(_) => debug!("File deleted successfully: {}", archived_file.display()),
                Err(e) => error!("Error deleting file: {}", e),
            };
        }
    }
    Ok(())
}

/// Deletes a single file if it is older than the specified period in days.
///
/// The function first checks if the file's creation date is in the future,
/// and if so, skips the file. Otherwise, it calculates the number of days
/// since the file was created and compares it to the specified period.
/// If the file is older than the specified period, it is deleted.
///
/// # Arguments
///
/// * `file` - A reference to the `PathBuf` of the file to be deleted.
/// * `period` - The period in days beyond which the file should be deleted.
///
/// # Errors
///
/// Returns an error if the file's creation date cannot be determined,
/// if the file is dated in the future, or if the file cannot be deleted.
fn delete_expired_file(file: &PathBuf, period: usize) -> Result<(), Box<dyn Error>> {
    debug!("Called: Client::delete_expired_file; file: {:?} period: {}", file, period);
    let now = SystemTime::now();
    debug!("Current time: {:?}", now);
    let file_datetime = fs::metadata(file)?.modified()?;
    debug!("File creation time: {:?}", file_datetime);
    let duration = now.duration_since(file_datetime)?;
    let days = duration.as_secs() / SECS_PER_DAY;
    debug!("Days since file creation: {}", days);

    // Delete the file if it exceeds the specified period
    if days > period as u64 {
        debug!("File {} is older than {} days, deleting", file.display(), period);
        debug!("Deleting file: {}", file.display());
        fs::remove_file(file)?;
        info!("File {} deleted", file.display());
    } else {
        debug!("File {} is not older than {} days, skipping", file.display(), period);
    }

    Ok(())
}

/// Returns the title of the file based on its filename.
///
/// The title is the filename without its extension, truncated to a maximum length of
/// `MAX_TITLE_LENGTH`. This is used as the title when uploading the file to Paperless-ngx.
fn get_title_from_filename(file: &std::path::Path) -> String {
    let file_name = match file.file_name() {
        Some(name) => name.to_str().unwrap(),
        None => "",
    };
    let extension = match file.extension() {
        Some(ext) => format!(".{}", ext.to_str().unwrap()),
        None => "".to_string(),
    };
    let name = file_name.replace(extension.as_str(), "");
    // Truncate the name to a maximum length of `MAX_TITLE_LENGTH`.
    // This is a safety precaution to avoid overflowing the Paperless-ngx title field.
    let title = name.split_at(min(name.len(), MAX_TITLE_LENGTH)).0;
    title.into()
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use log::LevelFilter::Debug;

    fn setup_logger() {
        let _ = env_logger::builder().filter_level(Debug).is_test(true).try_init();
    }

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

    #[test]
    fn test_upload_file_success() {
        setup_logger();

        let mut _s = mockito::Server::new();
        let _m = _s.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(http::StatusCode::OK.as_u16() as usize)
            .create();

        let mut cfg = Config::default();
        cfg.private_config.token = "token".to_string();
        cfg.public_config.endpoint = format!("{}/api/endpoint", _s.url());
        let client = Client::new(cfg).unwrap();

        let file_path = PathBuf::from("test_file.txt");
        fs::File::create(&file_path).unwrap();

        assert!(client.upload_file(&file_path).is_ok());
        _m.assert();

        // Delete the test file if possible
        // Ignore any errors
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_upload_file_error_creating_form() {
        setup_logger();

        let cfg = Config::default();
        let client = Client::new(cfg).unwrap();

        let file_path = PathBuf::from("non_existent_file.txt");

        assert!(client.upload_file(&file_path).is_err());
    }

    #[test]
    fn test_upload_file_error_sending_request() {
        setup_logger();

        let mut _s = mockito::Server::new();
        let _m = _s.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(http::StatusCode::INTERNAL_SERVER_ERROR.as_u16() as usize)
            .create();

        let mut cfg = Config::default();
        cfg.private_config.token = "token".to_string();
        cfg.public_config.endpoint = format!("{}/api/endpoint", _s.url());
        let client = Client::new(cfg).unwrap();

        let file_path = PathBuf::from("test_file.txt");
        fs::File::create(&file_path).unwrap();

        assert!(client.upload_file(&file_path).is_err());
        _m.assert();

        // Delete the test file if possible
        // Ignore any errors
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_upload_file_non_ok_status() {
        setup_logger();

        let mut _s = mockito::Server::new();
        let _m = _s.mock("POST", "/api/endpoint")
            .match_header("Authorization", "Token token")
            .with_status(http::StatusCode::BAD_REQUEST.as_u16() as usize)
            .create();

        let mut cfg = Config::default();
        cfg.private_config.token = "token".to_string();
        cfg.public_config.endpoint = format!("{}/api/endpoint", _s.url());
        let client = Client::new(cfg).unwrap();


        let file_path = PathBuf::from("test_file.txt");
        fs::File::create(&file_path).unwrap();

        assert!(client.upload_file(&file_path).is_err());
        _m.assert();

        // Delete the test file if possible
        // Ignore any errors
        let _ = fs::remove_file(file_path);
    }
}