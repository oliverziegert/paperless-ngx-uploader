//! File operations for document handling.
//!
//! This module contains functionality for file operations such as
//! archiving and deletion of processed documents.

use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

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
