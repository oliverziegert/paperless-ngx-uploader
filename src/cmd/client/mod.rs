// Module coordinator for the client functionality
// This module separates HTTP client logic, file operations, and helper functions

mod file_ops;
mod helpers;
mod http;

#[cfg(test)]
mod tests;

// Re-export Client and UploadOptions publicly so they can be used from main.rs
pub use http::{Client, UploadOptions};
