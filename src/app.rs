use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::audio::AudioSystem;
use crate::state::{AppState, AudioControl, PlaybackStatus, Track};

pub struct App {
    state: AppState,
    audio_system: AudioSystem,
}

impl App {
    pub fn new(root_dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut state = AppState::new();

        // Initial library load
        let tracks = Self::load_tracks_from_directory(&root_dir)?;
        state.library.tracks = tracks;
        state.library.current_dir = root_dir;

        Ok(Self {
            audio_system: AudioSystem::new(&state),
            state,
        })
    }

    fn load_tracks_from_directory(dir: &PathBuf) -> Result<Vec<Track>, Box<dyn Error>> {
        let mut tracks = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "mp3") {
                tracks.push(Track::new(path));
            }
        }

        Ok(tracks)
    }

    pub fn toggle_playback(&mut self) -> Result<(), Box<dyn Error>> {
        match self.state.playback.status {
            PlaybackStatus::Playing => {
                self.audio_system.pause();
            }
            PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                if self.state.playback.current_track.is_some() {
                    self.audio_system.resume();
                } else if !self.state.library.tracks.is_empty() {
                    // Default to first track if no track selected
                    self.select_track(0)?;
                }
            }
        }
        Ok(())
    }

    pub fn select_track(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        if index < self.state.library.tracks.len() {
            // Stop any current playback
            self.audio_system.stop();

            // Set the current track in state
            self.state.set_current_track(index);

            // Play the new track
            self.audio_system.play_track();
        }
        Ok(())
    }

    pub fn update(&mut self, delta: Duration) {
        // Update spectrum data for visualization
        let spectrum_data = self.audio_system.get_spectrum_data();
        self.state.update_spectrum(spectrum_data);

        // Update playback time
        self.audio_system.update_playback(delta);

        // Sync audio controls
        self.audio_system.update_audio_controls();
    }

    // Getter for UI to access state
    pub fn get_state(&self) -> &AppState {
        &self.state
    }
}
impl App {
    // Add these methods to handle keybindings
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool, Box<dyn Error>> {
        match key_event.code {
            // Playback controls
            KeyCode::Enter => {
                if let Some(selected_index) = self.state.library.selected_index {
                    self.select_track(selected_index)?;
                }
            }
            KeyCode::Char('p') => {
                self.toggle_playback()?;
            }
            // Navigation in track list
            KeyCode::Char('k') => {
                self.move_selection_up();
            }
            KeyCode::Char('j') => {
                self.move_selection_down();
            }
            // Audio controls
            KeyCode::Left => {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    self.state
                        .set_audio_control(AudioControl::Balance, self.state.audio.balance - 5.0);
                } else {
                    self.state
                        .set_audio_control(AudioControl::Volume, self.state.audio.volume - 5.0);
                }
            }
            KeyCode::Right => {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    self.state
                        .set_audio_control(AudioControl::Balance, self.state.audio.balance + 5.0);
                } else {
                    self.state
                        .set_audio_control(AudioControl::Volume, self.state.audio.volume + 5.0);
                }
            }
            KeyCode::Up => {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    self.state
                        .set_audio_control(AudioControl::Treble, self.state.audio.treble + 5.0);
                } else {
                    self.state
                        .set_audio_control(AudioControl::Bass, self.state.audio.bass + 5.0);
                }
            }
            KeyCode::Down => {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    self.state
                        .set_audio_control(AudioControl::Treble, self.state.audio.treble - 5.0);
                } else {
                    self.state
                        .set_audio_control(AudioControl::Bass, self.state.audio.bass - 5.0);
                }
            }
            KeyCode::Char('q') => return Ok(false), // Signal to exit
            _ => {}
        }
        Ok(true)
    }

    fn move_selection_up(&mut self) {
        if let Some(current_index) = self.state.library.selected_index {
            if current_index > 0 {
                self.state.library.selected_index = Some(current_index - 1);
            }
        }
    }

    fn move_selection_down(&mut self) {
        if let Some(current_index) = self.state.library.selected_index {
            if current_index < self.state.library.tracks.len() - 1 {
                self.state.library.selected_index = Some(current_index + 1);
            }
        }
    }
}
