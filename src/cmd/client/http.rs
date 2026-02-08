use crate::cmd::config::Config;
use crate::cmd::models::CmdError;
use http::header;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
use regex::Regex;
use reqwest::multipart;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::file_ops::{aggregate_files, archive_files, delete_expired_files};
use super::helpers::get_title_from_filename;

const HEADER_AUTH_PREFIX: &str = "Token ";

/// Options for configuring the upload operation.
///
/// Groups all upload-related parameters into a single struct for cleaner
/// function signatures and easier parameter management.
#[allow(clippy::struct_excessive_bools)]
pub struct UploadOptions {
    /// Optional path to a single file to upload
    pub file: Option<PathBuf>,
    /// Optional path to a folder containing files to upload
    pub folder: Option<PathBuf>,
    /// If true, recursively scan subfolders
    pub recursive: bool,
    /// Regex pattern to filter file names
    pub filter: String,
    /// If true, successfully uploaded files are archived
    pub archive: bool,
    /// Number of days after which files are considered expired (used with `delete`)
    pub period: usize,
    /// If true, files older than `period` days are deleted
    pub delete: bool,
    /// If true, simulates upload without making actual changes
    pub dry_run: bool,
}

pub struct Client {
    pub(super) cfg: Config,
    http: reqwest::Client,
}

/// Statistics collected during the upload operation.
///
/// Tracks counts for all file operations performed during the upload process,
/// including successful and failed uploads, filtered files, and post-upload
/// operations like archival and deletion.
pub struct UploadStatistics {
    /// Total number of files found matching the filter criteria
    pub total_found: usize,
    /// Number of files successfully uploaded
    pub uploaded_successfully: usize,
    /// Number of files that failed to upload
    pub upload_failed: usize,
    /// Number of files skipped (filtered out)
    pub skipped: usize,
    /// Number of files successfully archived
    pub archived: usize,
    /// Number of files successfully deleted
    pub deleted: usize,
}

/// Displays a summary of the upload operation statistics.
///
/// Logs a formatted summary of all file operations performed during the upload,
/// including files found, uploaded, failed, archived, and deleted.
///
/// # Arguments
///
/// * `stats` - Reference to the upload statistics to display
fn display_summary(stats: &UploadStatistics) {
    info!("Upload Summary:");
    info!("  Total files found: {}", stats.total_found);
    info!("  Successfully uploaded: {}", stats.uploaded_successfully);
    info!("  Failed uploads: {}", stats.upload_failed);
    info!("  Files skipped: {}", stats.skipped);
    info!("  Files archived: {}", stats.archived);
    info!("  Files deleted: {}", stats.deleted);
}

impl Client {
    /// Creates a new `Client` with the given `Config`.
    ///
    /// The `Config` is used to create a `reqwest::Client` with the
    /// `Authorization` header set to the token configured in the `Config`.
    ///
    /// # Errors
    ///
    /// Returns `Err(CmdError::ClientCreationFailed)` if the HTTP client cannot be created.
    pub fn new(cfg: Config) -> Result<Self, CmdError> {
        debug!("Creating new Client with provided Config");

        let mut header = header::HeaderMap::new();
        debug!("HeaderMap created");

        let header_auth_value = format!("{}{}", HEADER_AUTH_PREFIX, cfg.private_config.token);
        let mut header_auth_value = header::HeaderValue::from_str(header_auth_value.as_str())
            .map_err(|_| CmdError::ClientCreationFailed)?;
        header_auth_value.set_sensitive(true);
        header.insert(header::AUTHORIZATION, header_auth_value);
        debug!("Authorization header set");

        header.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));
        debug!("Accept header set");

        let client = reqwest::Client::builder()
            .default_headers(header)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| CmdError::ClientCreationFailed)?;
        debug!("HTTP Client created successfully");

        Ok(Self { cfg, http: client })
    }

    /// Checks connectivity and authentication with the Paperless-ngx API.
    ///
    /// Makes a GET request to the /api/ endpoint to verify that the configured
    /// endpoint is reachable and the authentication token is valid.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network connection fails
    /// - Authentication fails (401 Unauthorized)
    /// - Server returns an error status code
    /// - The API endpoint is not accessible
    pub async fn check_status(&self) -> Result<(), Box<dyn Error>> {
        debug!("Called: Client::check_status");

        // Construct the API endpoint URL
        let api_url = format!("{}/api/", self.cfg.public_config.endpoint.trim_end_matches('/'));
        debug!("Checking status at: {api_url}");

        // Make GET request to /api/ endpoint
        let response = match self.http.get(&api_url).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to connect to Paperless-ngx API: {e}");
                return Err(e.into());
            }
        };

        let status = response.status();
        debug!("Response status: {status}");

        if status == reqwest::StatusCode::OK {
            info!("Successfully connected to Paperless-ngx API");
            info!("Endpoint: {}", self.cfg.public_config.endpoint);
            info!("Authentication: Valid");
            Ok(())
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            error!("Authentication failed: Invalid or missing token");
            Err("Authentication failed: 401 Unauthorized".into())
        } else {
            error!("API returned error status: {status}");
            Err(format!("API returned error status: {status}").into())
        }
    }

    /// Uploads documents to Paperless-ngx with optional archival and cleanup.
    ///
    /// This method aggregates files from either a single file or a folder, filters
    /// them by the provided regex pattern, uploads them to the Paperless-ngx endpoint,
    /// and optionally archives uploaded files and deletes expired files.
    ///
    /// # Arguments
    ///
    /// * `options` - Upload configuration options (see `UploadOptions`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File aggregation fails (invalid regex, folder not readable)
    /// - File upload fails (network error, authentication failure, server error)
    /// - Archival fails (unable to create archive folder, file move error)
    /// - Deletion fails (unable to delete expired files)
    pub async fn upload(&self, options: UploadOptions) -> Result<(), Box<dyn Error>> {
        debug!("Called: Client::upload");
        let filter = Regex::new(&options.filter)?;
        let files = &aggregate_files(options.file, options.folder, options.recursive, &filter)?;

        if files.is_empty() {
            info!("No files found. Nothing to do.");
            return Ok(());
        }

        // Initialize statistics
        let mut stats = UploadStatistics {
            total_found: files.len(),
            uploaded_successfully: 0,
            upload_failed: 0,
            skipped: 0,
            archived: 0,
            deleted: 0,
        };

        if options.dry_run {
            info!("DRY RUN MODE - No actual changes will be made");
            info!("Would upload {} files:", files.len());
            for file in files {
                let title = get_title_from_filename(file);
                info!("  - {title}");
            }
            stats.uploaded_successfully = files.len();

            if options.archive {
                info!("Would archive {} files after successful upload", files.len());
                stats.archived = files.len();
            }

            if options.delete {
                info!("Would delete files older than {} days", options.period);
                // In dry-run mode, we don't actually check which files would be deleted
                // to avoid complexity, just note that the operation would run
            }
        } else {
            let (files_to_archive, successful_count, failed_count) =
                match self.upload_files(files).await {
                    Ok(result) => result,
                    Err(e) => {
                        error!("Error uploading files: {e}");
                        return Err(e);
                    }
                };

            stats.uploaded_successfully = successful_count;
            stats.upload_failed = failed_count;
            stats.skipped = stats.total_found - stats.uploaded_successfully - stats.upload_failed;

            if options.archive {
                let archived_count = archive_files(&files_to_archive);
                stats.archived = archived_count;
            }

            if options.delete {
                let deleted_count = delete_expired_files(files, options.period)?;
                stats.deleted = deleted_count;
            }
        }

        display_summary(&stats);

        Ok(())
    }

    /// Uploads a list of files to Paperless-ngx in parallel.
    ///
    /// This method spawns concurrent tasks to upload files in parallel using
    /// `tokio::spawn` and `JoinSet`. Successfully uploaded files are collected and
    /// returned. If an individual file upload fails, the error is logged and
    /// the method continues with the remaining files.
    ///
    /// # Arguments
    ///
    /// * `files` - A vector of file paths to upload
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// * A vector of paths for files that were successfully uploaded
    /// * The count of successful uploads
    /// * The count of failed uploads
    ///
    /// # Errors
    ///
    /// This method does not return errors. Individual file upload failures are
    /// logged but do not stop the upload process.
    pub(super) async fn upload_files(
        &self,
        files: &[PathBuf],
    ) -> Result<(Vec<PathBuf>, usize, usize), Box<dyn Error>> {
        use std::sync::Arc;
        use tokio::sync::Semaphore;
        use tokio::task::JoinSet;

        const MAX_CONCURRENT_UPLOADS: usize = 10;

        debug!("Called: Client::upload_files");

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_UPLOADS));
        let mut set = JoinSet::new();

        // Spawn a task for each file upload
        for file in files {
            let file = file.clone();
            let http_client = self.http.clone();
            let endpoint = self.cfg.public_config.endpoint.clone();
            let permit = semaphore.clone().acquire_owned().await?;

            set.spawn(async move {
                let result = Self::upload_file_task(http_client, endpoint, &file).await;
                drop(permit); // Release semaphore permit when upload completes
                (file, result)
            });
        }

        // Initialize progress bar
        let progress_bar = ProgressBar::new(files.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg} (ETA: {eta})")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("=>-"),
        );

        // Collect results from all tasks
        let mut files_archived: Vec<PathBuf> = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;
        while let Some(result) = set.join_next().await {
            match result {
                Ok((file, Ok(()))) => {
                    debug!("File uploaded successfully");
                    let file_name = file.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    progress_bar.set_message(format!("Uploaded: {file_name}"));
                    files_archived.push(file);
                    successful_count += 1;
                    progress_bar.inc(1);
                }
                Ok((file, Err(e))) => {
                    let file_display = file.display();
                    error!("Error uploading file {file_display}: {e}");
                    let file_name = file.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    progress_bar.set_message(format!("Failed: {file_name}"));
                    failed_count += 1;
                    progress_bar.inc(1);
                }
                Err(e) => {
                    error!("Task join error: {e}");
                    progress_bar.set_message("Task failed");
                    failed_count += 1;
                    progress_bar.inc(1);
                }
            }
        }

        progress_bar.finish_with_message("Upload complete");

        Ok((files_archived, successful_count, failed_count))
    }

    /// Helper method to upload a single file within a spawned task.
    ///
    /// This is a static method that can be called from spawned tasks without
    /// borrowing self. It performs the same upload logic but
    /// takes owned parameters.
    async fn upload_file_task(
        http_client: reqwest::Client,
        endpoint: String,
        file: &PathBuf,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        debug!("Called: Client::upload_file_task");

        // Read file content synchronously
        let file_content = match fs::read(file) {
            Ok(content) => content,
            Err(e) => {
                error!("Error reading file: {e}");
                return Err(e.into());
            }
        };

        let title = get_title_from_filename(file);

        // Create multipart form with file content
        let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("document");

        let part = multipart::Part::bytes(file_content).file_name(file_name.to_string());

        let form = multipart::Form::new().part("document", part).text("title", title.clone());

        debug!("file_name: {}", &title);

        let response = match http_client.post(&endpoint).multipart(form).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Error uploading file: {e}");
                return Err(e.into());
            }
        };

        let status = response.status();
        if status == reqwest::StatusCode::OK {
            info!("File {title} uploaded successfully");
            Ok(())
        } else {
            error!("Error uploading file: {status}");
            Err(format!("Error uploading file: {status}").into())
        }
    }
}
