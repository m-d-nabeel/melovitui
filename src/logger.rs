use chrono::Local;
use log::{error, info, warn, LevelFilter};
use log4rs::{
    append::rolling_file::{
        policy::compound::{
            roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
        },
        RollingFileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::fd::AsRawFd,
    panic,
    sync::Mutex,
};

// Global file handle for panic logging
lazy_static::lazy_static! {
    static ref PANIC_FILE: Mutex<File> = Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/panic.log")
            .expect("Failed to open panic log file")
    );
}

pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory
    std::fs::create_dir_all("logs")?;

    // Set up regular logging
    let file_appender = Box::new(
        RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{d(%Y-%m-%d %H:%M:%S)} - {l} - {m}\n",
            )))
            .build(
                "logs/app.log",
                Box::new(CompoundPolicy::new(
                    Box::new(SizeTrigger::new(10 * 1024 * 1024)), // 10MB
                    Box::new(FixedWindowRoller::builder().build("logs/app.{}.log", 5)?),
                )),
            )?,
    );

    let config = Config::builder()
        .appender(Appender::builder().build("file", file_appender))
        .build(Root::builder().appender("file").build(LevelFilter::Info))?;

    // Initialize regular logging
    log4rs::init_config(config)?;

    // Set up panic handler
    setup_panic_handler();

    // Set up stderr redirection
    redirect_stderr()?;

    info!("Logging system initialized");
    Ok(())
}

fn setup_panic_handler() {
    panic::set_hook(Box::new(|panic_info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unknown");
        let backtrace = std::backtrace::Backtrace::capture();

        let panic_message = format!(
            "\n[PANIC] {} - Thread: {}\nInfo: {:?}\nBacktrace:\n{:?}\n",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            thread_name,
            panic_info,
            backtrace
        );

        // Log to panic file
        if let Ok(mut file) = PANIC_FILE.lock() {
            let _ = file.write_all(panic_message.as_bytes());
            let _ = file.flush();
        }

        // Also log using regular logger if available
        error!("PANIC: {}", panic_message);
    }));
}

fn redirect_stderr() -> Result<(), Box<dyn std::error::Error>> {
    let stderr_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/stderr.log")?;

    std::env::set_var("RUST_BACKTRACE", "1");

    unsafe {
        if libc::dup2(stderr_file.as_raw_fd(), std::io::stderr().as_raw_fd()) != -1 {
            info!("Successfully redirected stderr to file");
        } else {
            warn!("Failed to redirect stderr to file");
        }
    }

    Ok(())
}
