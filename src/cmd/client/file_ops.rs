//! File operations for document handling.
//!
//! This module contains functionality for file operations such as
//! archiving and deletion of processed documents.

use crate::cmd::models::CmdError;
use log::{debug, error, info};
use regex::Regex;
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
/// - If `recursive` is true, subdirectories will be traversed recursively.
/// - The `filter` argument is a pre-compiled regex used to filter file names.
///
/// Returns a vector of `PathBuf` on success, or an error if the folder cannot be read.
pub fn aggregate_files(
    file: Option<PathBuf>,
    folder: Option<PathBuf>,
    recursive: bool,
    filter: &Regex,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths = Vec::new();

    if let Some(folder_path) = folder {
        if recursive {
            collect_files_recursive(&folder_path, filter, &mut paths)?;
        } else {
            let entries = fs::read_dir(folder_path)?;
            for entry in entries {
                let entry_path = entry?.path();
                if entry_path.is_file() {
                    if let Some(file_name) = entry_path.file_name() {
                        if let Some(file_name_str) = file_name.to_str() {
                            if filter.is_match(file_name_str) {
                                paths.push(entry_path);
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(file_path) = file {
        if let Some(file_name) = file_path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                if filter.is_match(file_name_str) {
                    paths.push(file_path);
                }
            }
        }
    }

    Ok(paths)
}

/// Recursively collects files from a directory and its subdirectories.
///
/// This helper function traverses directories recursively, applying the regex filter
/// to all files found at any depth level.
///
/// # Arguments
///
/// * `dir` - The directory to traverse
/// * `filter` - A pre-compiled regex used to filter file names
/// * `paths` - A mutable vector to collect matching file paths
///
/// # Errors
///
/// Returns an error if any directory cannot be read.
fn collect_files_recursive(
    dir: &PathBuf,
    filter: &Regex,
    paths: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry_path = entry?.path();
        if entry_path.is_dir() {
            collect_files_recursive(&entry_path, filter, paths)?;
        } else if entry_path.is_file() {
            if let Some(file_name) = entry_path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if filter.is_match(file_name_str) {
                        paths.push(entry_path);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Archives a list of files based on the provided period.
///
/// This function iterates over the provided files and archives each one
/// if its age exceeds the specified period in days.
///
/// # Arguments
///
/// * `files` - A reference to a vector of `PathBuf` representing the files to be archived.
///
/// # Returns
///
/// Returns the count of successfully archived files.
pub fn archive_files(files: &[PathBuf]) -> usize {
    debug!("Starting archive_files");
    let mut archived_count = 0;
    for file in files {
        let file_display = file.display();
        debug!("Attempting to archive file: {file_display}");
        match archive_file(file) {
            Ok(()) => {
                debug!("File archived successfully: {file_display}");
                archived_count += 1;
            }
            Err(e) => error!("Error archiving file {file_display}: {e}"),
        }
    }
    debug!("Completed archive_files with {archived_count} files archived");
    archived_count
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
///
/// # Errors
///
/// Returns an error if any file operation fails during the archiving process.
fn archive_file(file: &PathBuf) -> Result<(), Box<dyn Error>> {
    debug!("Called: Client::archive_file");
    let parent = file
        .parent()
        .ok_or_else(|| CmdError::InvalidFilePath(file.display().to_string()))?;
    let archive_folder = parent.join(ARCHIVE_FOLDER_NAME);
    let archive_display = archive_folder.display();
    debug!("Creating archive folder: {archive_display}");
    fs::create_dir_all(&archive_folder)?;
    let file_name = file
        .file_name()
        .ok_or_else(|| CmdError::InvalidFilePath(file.display().to_string()))?;
    let target_path = archive_folder.join(file_name);
    let file_display = file.display();
    let target_display = target_path.display();
    debug!("Moving file {file_display} to {target_display}");

    // Move the file to the archive folder
    fs::rename(file, &target_path)?;
    info!("File {file_display} moved to archive folder");
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
/// # Returns
///
/// Returns the count of successfully deleted files.
///
/// # Errors
///
/// Logs an error if any file cannot be deleted.
pub fn delete_expired_files(files: &[PathBuf], period: usize) -> Result<usize, Box<dyn Error>> {
    debug!("Called: Client::delete_expired_files");
    let mut deleted_count = 0;
    // Iterate over each file in the list
    for file in files {
        let file_display = file.display();
        debug!("Attempting to delete archived files for: {file_display}");
        let parent = file
            .parent()
            .ok_or_else(|| CmdError::InvalidFilePath(file.display().to_string()))?;
        let archive_folder = parent.join(ARCHIVE_FOLDER_NAME);
        if !archive_folder.is_dir() {
            let archive_display = archive_folder.display();
            debug!("Archive folder does not exist: {archive_display}");
            continue;
        }
        let archived_files = fs::read_dir(&archive_folder)?.collect::<Vec<_>>();
        for archived_file in archived_files {
            let archived_file = archived_file?.path();
            match delete_expired_file(&archived_file, period) {
                Ok(()) => {
                    let archived_display = archived_file.display();
                    debug!("File deleted successfully: {archived_display}");
                    deleted_count += 1;
                }
                Err(e) => error!("Error deleting file: {e}"),
            };
        }
    }
    debug!("Completed delete_expired_files with {deleted_count} files deleted");
    Ok(deleted_count)
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
    let file_display = file.display();
    debug!("Called: Client::delete_expired_file; file: {file_display} period: {period}");
    let now = SystemTime::now();
    debug!("Current time: {now:?}");
    let file_datetime = fs::metadata(file)?.modified()?;
    debug!("File creation time: {file_datetime:?}");
    let duration = now.duration_since(file_datetime)?;
    let days = duration.as_secs() / SECS_PER_DAY;
    debug!("Days since file creation: {days}");

    // Delete the file if it exceeds the specified period
    let file_display = file.display();
    if days > period as u64 {
        debug!("File {file_display} is older than {period} days, deleting");
        debug!("Deleting file: {file_display}");
        fs::remove_file(file)?;
        info!("File {file_display} deleted");
    } else {
        debug!("File {file_display} is not older than {period} days, skipping");
    }

    Ok(())
}
