//! File operations for document handling.
//!
//! This module contains functionality for file operations such as
//! archiving and deletion of processed documents.

use crate::cmd::models::CmdError;
use log::{debug, error, info};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

pub const ARCHIVE_FOLDER_NAME: &str = "archive";
const SECS_PER_DAY: u64 = 60 * 60 * 24;

/// Combines file and folder paths into a single vector.
///
/// - If the `file` argument is provided, it adds the file to the vector.
/// - If the `folder` argument is provided, it reads all entries in the folder
///   and adds them to the vector.
/// - The `filter` argument is a regex pattern used to filter file names.
///
/// Returns a vector of `PathBuf` on success, or an error if the folder cannot be read.
pub fn aggregate_files(
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
pub fn archive_files(files: &[PathBuf]) -> Result<(), Box<dyn Error>> {
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
pub fn delete_expired_files(files: &[PathBuf], period: usize) -> Result<(), Box<dyn Error>> {
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
