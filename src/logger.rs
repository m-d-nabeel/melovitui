use chrono::Local;
use log::{debug, error, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use std::{fs::create_dir_all, panic};

/// Configure a simple logging system with separate debug and error logs
pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory
    create_dir_all("logs")?;

    // Debug logs - includes all levels (debug, info, warn)
    let debug_log = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}{n}",
        )))
        .build("logs/debug.log")?;

    // Error logs - only errors
    let error_log = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} [{l}] [{f}:{L}] {m}{n}",
        )))
        .build("logs/error.log")?;

    // Set the appropriate log level based on compilation profile
    let root_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Off
    };

    // Build log configuration
    let config = Config::builder()
        .appender(Appender::builder().build("debug_file", Box::new(debug_log)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Error)))
                .build("error_file", Box::new(error_log)),
        )
        .logger(
            Logger::builder()
                .appender("error_file")
                .build("error", LevelFilter::Error),
        )
        .build(
            Root::builder()
                .appender("debug_file")
                .appender("error_file")
                .build(root_level),
        )?;

    // Initialize logging
    log4rs::init_config(config)?;

    // Set up panic handler
    setup_panic_handler();

    if cfg!(debug_assertions) {
        debug!("Logging system initialized with debug level");
    } else {
        debug!("Logging system initialized with info level (debug logs disabled)");
    }
    Ok(())
}

fn setup_panic_handler() {
    panic::set_hook(Box::new(|panic_info| {
        // Try to restore terminal state on panic
        let _ = ratatui::crossterm::terminal::disable_raw_mode();
        let _ = ratatui::crossterm::execute!(
            std::io::stdout(),
            ratatui::crossterm::terminal::LeaveAlternateScreen,
            ratatui::crossterm::cursor::Show
        );

        // Basic panic information
        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_else(|| "unknown location".to_string());

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

        // Log the panic
        error!(
            "APPLICATION PANIC at {} - Location: {} - Info: {:?}",
            timestamp, location, panic_info
        );

        // In a TUI application, ensure the terminal is restored
        std::process::exit(1);
    }));
}

// Convenient logging macros that include context
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            log::debug!(
                "[{}:{}] {}",
                file!(),
                line!(),
                format!($($arg)*)
            )
        }
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        log::error!(
            "[{}:{}] {}",
            file!(),
            line!(),
            format!($($arg)*)
        )
    }
}
