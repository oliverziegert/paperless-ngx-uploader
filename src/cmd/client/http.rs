use crate::cmd::config::Config;
use crate::cmd::models::CmdError;
use http::header;
use log::{debug, error, info};
use reqwest::multipart;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use super::file_ops::{aggregate_files, archive_files, delete_expired_files};
use super::helpers::get_title_from_filename;

const HEADER_AUTH_PREFIX: &str = "Token ";


pub struct Client {
    pub(super) cfg: Config,
    http: reqwest::Client,
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

        header.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        debug!("Accept header set");

        let client = reqwest::Client::builder()
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
        let files = &aggregate_files(file, folder, filter)?;

        if files.is_empty() {
            info!("No files found. Nothing to do.");
            return Ok(());
        }

        let files_to_archive = match self.upload_files(files).await {
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
    async fn upload_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        debug!("Called: Client::upload_files");
        let mut files_archived: Vec<PathBuf> = Vec::new();
        for file in files.iter() {
            match self.upload_file(file).await {
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
    pub(super) async fn upload_file(&self, file: &PathBuf) -> Result<(), Box<dyn Error>> {
        debug!("Called: Client::upload_file");

        // Read file content synchronously
        let file_content = match fs::read(file) {
            Ok(content) => content,
            Err(e) => {
                error!("Error reading file: {}", e);
                return Err(e.into());
            }
        };

        let title = get_title_from_filename(file);

        // Create multipart form with file content
        let file_name = file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document");

        let part = multipart::Part::bytes(file_content)
            .file_name(file_name.to_string());

        let form = multipart::Form::new()
            .part("document", part)
            .text("title", title.clone());

        debug!("file_name: {}", &title);

        let response = match self.http.post(&self.cfg.public_config.endpoint).multipart(form).send().await {
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