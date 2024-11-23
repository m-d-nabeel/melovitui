use config::get_music_dir;

use std::{
    io,
    time::{Duration, Instant},
};

use app::App;
use ratatui::{
    crossterm::{
        cursor::{Hide, Show},
        event::{self, DisableMouseCapture, EnableMouseCapture, Event},
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
    logger::setup_logging()?;
    // Get music directory path
    let music_dir = get_music_dir();
    log::info!("Using music directory: {:?}", music_dir);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, DisableMouseCapture, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut ui_manager = UIManager::new();
    let mut terminal = Terminal::new(backend)?;

    // Create app state and initialize audio
    let mut app = App::new(music_dir)?;

    // Run main app loop
    let result = run_app(&mut terminal, &mut app, &mut ui_manager);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        EnableMouseCapture,
        Show
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("{:?}", err);
    }

    Ok(())
}
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    ui_manager: &mut UIManager,
) -> io::Result<()> {
    // Define the update interval (e.g., 16ms for ~60 FPS)
    let tick_rate = Duration::from_millis(16);
    let mut last_tick = Instant::now();

    loop {
        // Render UI
        terminal.draw(|f| ui_manager.render(f, app))?;

        // Handle timing for updates
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Poll for events with a timeout
        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                match app.handle_key_event(key_event) {
                    Ok(true) => continue,
                    Ok(false) => return Ok(()), // Exit app
                    Err(e) => {
                        eprintln!("Error handling key event: {:?}", e);
                    }
                }
            }
        }

        // Check if we should update the app state
        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }
    }
}
