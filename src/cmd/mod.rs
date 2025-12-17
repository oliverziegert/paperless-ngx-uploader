pub mod config;
pub mod logger;
pub mod input;
pub mod client;
mod models;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");