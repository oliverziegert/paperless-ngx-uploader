use colored::Colorize;
use log::{debug, LevelFilter};
use std::io::Write;

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

    // Set custom format with colorized output
    log_builder.format(|buf, record| {
        let level = record.level();
        let level_str = format!("{:5}", level);

        let colored_level = match level {
            log::Level::Error => level_str.red().bold(),
            log::Level::Warn => level_str.yellow().bold(),
            log::Level::Info => level_str.green().bold(),
            log::Level::Debug => level_str.cyan().dimmed(),
            log::Level::Trace => level_str.white().dimmed(),
        };

        writeln!(buf, "{} {}", colored_level, record.args())
    });

    // Initialize the logger
    log_builder.init();
    debug!("Called: setup_logging; verbose: {verbose}");
    debug!("Log level: {:?}", log::max_level());
}
