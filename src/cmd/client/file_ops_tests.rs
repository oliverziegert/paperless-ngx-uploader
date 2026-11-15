//! Tests for the cleanup of the `archive/` folder.

use super::file_ops::{delete_expired_files, ARCHIVE_FOLDER_NAME};
use std::fs::{self, File};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

const DAY: Duration = Duration::from_secs(60 * 60 * 24);
const PERIOD: usize = 31;

/// Creates a temp directory with an `archive/` subfolder.
///
/// # Returns
///
/// The directory guard and the path of a (not existing) uploaded file in it,
/// which is what `delete_expired_files` takes to locate `archive/`.
fn setup() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(ARCHIVE_FOLDER_NAME)).unwrap();
    let uploaded = dir.path().join("uploaded.pdf");
    (dir, uploaded)
}

/// Creates a file in `archive/` with the given modification time.
///
/// # Arguments
///
/// * `dir` - The directory that holds the `archive/` subfolder.
/// * `name` - The file name.
/// * `modified` - The modification time to set.
///
/// # Returns
///
/// The path of the created file.
fn archived_file(dir: &TempDir, name: &str, modified: SystemTime) -> PathBuf {
    let path = dir.path().join(ARCHIVE_FOLDER_NAME).join(name);
    File::create(&path).unwrap().set_modified(modified).unwrap();
    path
}

#[test]
fn test_old_file_is_removed_and_counted() {
    let (dir, uploaded) = setup();
    let old = archived_file(&dir, "old.pdf", SystemTime::now() - 40 * DAY);

    let count = delete_expired_files(&[uploaded], PERIOD).unwrap();

    assert_eq!(count, 1);
    assert!(!old.exists());
}

#[test]
fn test_recent_file_is_kept_and_not_counted() {
    let (dir, uploaded) = setup();
    let recent = archived_file(&dir, "recent.pdf", SystemTime::now() - 5 * DAY);

    let count = delete_expired_files(&[uploaded], PERIOD).unwrap();

    assert_eq!(count, 0);
    assert!(recent.exists());
}

#[test]
fn test_file_exactly_as_old_as_period_is_kept() {
    let (dir, uploaded) = setup();
    let edge = archived_file(&dir, "edge.pdf", SystemTime::now() - 31 * DAY);

    let count = delete_expired_files(&[uploaded], PERIOD).unwrap();

    assert_eq!(count, 0);
    assert!(edge.exists());
}

#[test]
fn test_future_file_is_kept_and_does_not_stop_cleanup() {
    let (dir, uploaded) = setup();
    let future = archived_file(&dir, "future.pdf", SystemTime::now() + DAY);
    let old = archived_file(&dir, "old.pdf", SystemTime::now() - 40 * DAY);

    let count = delete_expired_files(&[uploaded], PERIOD).unwrap();

    assert_eq!(count, 1);
    assert!(future.exists());
    assert!(!old.exists());
}
