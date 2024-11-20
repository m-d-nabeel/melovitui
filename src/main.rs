use std::{io, time::Duration};

use app::App;
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::{Backend, CrosstermBackend},
    Terminal,
};
use ui::view::UIManager;

mod app;
mod audio;
mod controls;
mod state;
mod ui;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut ui_manager = UIManager::new();
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(Into::into("/home/m-d-nabeel/Music/"))?;
    // Create app state and initialize audio

    // Run main app loop
    let result = run_app(&mut terminal, &mut app, &mut ui_manager);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
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
    loop {
        // Render
        terminal.draw(|f| ui_manager.render(f, app))?;
        // Event handling
        if let Event::Key(key_event) = event::read()? {
            match app.handle_key_event(key_event) {
                Ok(true) => continue,
                Ok(false) => return Ok(()), // Exit app
                Err(e) => {
                    // Handle or log error
                    eprintln!("Error handling key event: {:?}", e);
                }
            }
        }
        // Update app state periodically
        app.update(Duration::from_millis(200));
    }
}
