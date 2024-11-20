use rodio::Decoder;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;

use crate::audio::AudioSystem;
use crate::state::AppState;

pub struct App {
    state: Arc<Mutex<AppState>>,
    audio_system: AudioSystem,
}

impl App {
    pub fn new(root_dir: String) -> Result<Self, Box<dyn Error>> {
        let app = Self {
            state: Arc::new(Mutex::new(AppState::new())),
            audio_system: AudioSystem { _stream, sink },
        };

        // Initial library load
        app.reload_library()?;
        Ok(app)
    }

    pub fn reload_library(&self) -> Result<(), Box<dyn Error>> {
        let mut state = self.state.lock().unwrap();
        state.ui.is_loading = true;

        // Load tracks synchronously
        let tracks = {
            // Your track loading logic here
            vec![] // Placeholder
        };

        state.library.tracks = tracks;
        state.ui.is_loading = false;
        Ok(())
    }

    pub fn toggle_playback(&self) -> Result<(), Box<dyn Error>> {
        let mut state = self.state.lock().unwrap();

        if state.playback.is_playing {
            self.audio_system.sink.pause();
        } else if let Some(current_index) = state.library.current_index {
            if let Some(track) = state.library.tracks.get(current_index) {
                let file = std::fs::File::open(&track.path)?;
                let source = Decoder::new(file)?;
                self.audio_system.sink.append(source);
                self.audio_system.sink.play();
            }
        }

        state.toggle_playback();
        Ok(())
    }

    // Getter for UI to access state
    pub fn get_state(&self) -> Arc<Mutex<AppState>> {
        Arc::clone(&self.state)
    }
}
