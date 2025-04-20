use config::get_music_dir;

use std::{
    io,
    time::{Duration, Instant},
};

use app::App;
use ratatui::{
    crossterm::{
        cursor::{Hide, Show},
        event::{self, DisableMouseCapture, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::{Backend, CrosstermBackend},
    Terminal,
};
use ui::view::UIManager;

mod app;
mod audio_system;
mod config;
mod controls;
mod logger;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging first, before any other operations
    if let Err(e) = logger::setup_logging() {
        // Don't print to console, it would clutter the TUI
        // Instead, we'll continue silently but without logging
        std::fs::create_dir_all("logs").ok();
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/init_error.log")
        {
            use std::io::Write;
            let _ = writeln!(&mut file, "Failed to setup logging system: {}", e);
        }
    }

    // Import macros after logger is initialized
    use crate::{log_debug, log_error};

    log_debug!("Application starting");

    // Setup terminal first to capture all later errors
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, DisableMouseCapture, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut ui_manager = UIManager::new();
    let mut terminal = Terminal::new(backend)?;

    log_debug!("Terminal UI initialized");

    // Get music directory and create app
    let dir = get_music_dir();
    let music_dir = {
        log_debug!("Using music directory: {:?}", dir);
        dir
    };

    // Create the app, handling any errors
    let app_result = App::new(music_dir);
    let mut app = match app_result {
        Ok(app) => {
            log_debug!("Application initialized successfully");
            app
        }
        Err(e) => {
            log_error!("Failed to initialize application: {:?}", e);
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, Show)?;
            terminal.show_cursor()?;

            return Err(e);
        }
    };

    // Run the application
    let result = run_app(&mut terminal, &mut app, &mut ui_manager);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, Show)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        // Log to file instead of printing to console
        log_error!("Application error: {:?}", err);
    } else {
        log_debug!("Application exited normally");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    ui_manager: &mut UIManager,
) -> io::Result<()> {
    use crate::{log_debug, log_error};

    log_debug!("Main application loop started");

    // Define the update interval (e.g., 16ms for ~60 FPS)
    let tick_rate = Duration::from_millis(16);
    let mut last_tick = Instant::now();

    loop {
        // Use a custom error handler for terminal drawing
        match terminal.draw(|f| ui_manager.render(f, app)) {
            Ok(_) => {}
            Err(e) => {
                log_error!("Terminal draw error: {:?}", e);
                // Don't interrupt the loop, try to continue
            }
        }

        // Handle timing for updates
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Poll for events with a timeout
        match event::poll(timeout) {
            Ok(true) => {
                if let Ok(Event::Key(key_event)) = event::read() {
                    match app.handle_key_event(key_event) {
                        Ok(true) => continue,
                        Ok(false) => {
                            log_debug!("Application exit requested");
                            return Ok(());
                        }
                        Err(e) => {
                            log_error!("Error handling key event: {:?}", e);
                            // Log but continue execution
                        }
                    }
                }
            }
            Ok(false) => {} // No event, continue with the update
            Err(e) => {
                log_error!("Event polling error: {:?}", e);
                // Small delay to prevent tight error loop
                std::thread::sleep(Duration::from_millis(100));
            }
        }

        // Check if we should update the app state
        if last_tick.elapsed() >= tick_rate {
            // Catch any panics in the update
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                app.update();
            })) {
                Ok(_) => {}
                Err(e) => {
                    let panic_msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic in app update".to_string()
                    };

                    log_error!("Recovered from panic in app update: {}", panic_msg);
                    // Try to continue execution
                }
            }

            last_tick = Instant::now();
        }
    }
}
