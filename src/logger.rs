use log4rs::{
    append::rolling_file::{
        policy::compound::{
            roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
        },
        RollingFileAppender,
    },
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};

pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure logs directory exists
    std::fs::create_dir_all("logs")?;

    // Create rolling file appender
    let file_appender = Box::new(
        RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}\n")))
            .build(
                "logs/app.log",
                Box::new(CompoundPolicy::new(
                    Box::new(SizeTrigger::new(10 * 1024 * 1024)), // 10MB
                    Box::new(FixedWindowRoller::builder().build("logs/app.{}.log", 5)?),
                )),
            )?,
    );

    // Create logging config
    let config = Config::builder()
        .appender(Appender::builder().build("file", file_appender))
        .logger(
            Logger::builder()
                .appender("file")
                .build("app", log::LevelFilter::Info),
        )
        .build(
            Root::builder()
                .appender("file")
                .build(log::LevelFilter::Info),
        )?;

    // Initialize the logger
    log4rs::init_config(config)?;

    Ok(())
}
