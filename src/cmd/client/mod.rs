// Module coordinator for the client functionality
// This module separates HTTP client logic, file operations, and helper functions

mod http;
mod file_ops;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export Client publicly so it can be used from main.rs
pub use http::Client;
