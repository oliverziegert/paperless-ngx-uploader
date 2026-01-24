pub mod config;
pub mod logger;
pub mod input;
pub mod client;
pub mod url_validator;
mod models;

const APP_NAME: &str = env!("CARGO_PKG_NAME");