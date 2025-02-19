use log::{debug, LevelFilter};

/// Set up the logging for the application
///
/// Takes a verbose level and sets the log level accordingly
pub fn setup_logging(verbose: u8) {
    let mut log_builder = env_logger::Builder::new();
    // Set global log level based on the verbose flag
    match verbose {
        0 => log_builder.filter_level(LevelFilter::Error),
        1 => log_builder.filter_level(LevelFilter::Warn),
        2 => log_builder.filter_level(LevelFilter::Info),
        3 => log_builder.filter_level(LevelFilter::Debug),
        _ => log_builder.filter_level(LevelFilter::Trace),
    };
    // Initialize the logger
    log_builder.init();
    debug!("Called: setup_logging; verbose: {}", verbose);
    debug!("Log level: {:?}", log::max_level());
}
