use crate::cmd::config::Config;
use crate::cmd::models::CmdError;
use http::header;
use log::{debug, error, info};
use reqwest::multipart;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::file_ops::{aggregate_files, archive_files, delete_expired_files};
use super::helpers::get_title_from_filename;

const HEADER_AUTH_PREFIX: &str = "Token ";

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
    pub async fn upload(
        &self,
        file: Option<PathBuf>,
        folder: Option<PathBuf>,
        filter: String,
        archive: bool,
        period: usize,
        delete: bool,
    ) -> Result<(), Box<dyn Error>> {
        debug!("Called: Client::upload");
        let files = &aggregate_files(file, folder, &filter)?;

        if files.is_empty() {
            info!("No files found. Nothing to do.");
            return Ok(());
        }

        let files_to_archive = match self.upload_files(files).await {
            Ok(files) => files,
            Err(e) => {
                error!("Error uploading files: {e}");
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
    /// Returns a vector of paths for files that were successfully uploaded.
    ///
    /// # Errors
    ///
    /// This method does not return errors. Individual file upload failures are
    /// logged but do not stop the upload process.
    pub(super) async fn upload_files(
        &self,
        files: &[PathBuf],
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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

        // Collect results from all tasks
        let mut files_archived: Vec<PathBuf> = Vec::new();
        while let Some(result) = set.join_next().await {
            match result {
                Ok((file, Ok(()))) => {
                    debug!("File uploaded successfully");
                    files_archived.push(file);
                }
                Ok((file, Err(e))) => {
                    let file_display = file.display();
                    error!("Error uploading file {file_display}: {e}");
                }
                Err(e) => {
                    error!("Task join error: {e}");
                }
            }
        }

        Ok(files_archived)
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
