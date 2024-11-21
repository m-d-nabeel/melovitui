use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::controls::audio::AudioSystem;
use crate::state::{AppState, AudioControl, PlaybackStatus, Track};

pub struct App {
    state: Arc<Mutex<AppState>>,
    audio_system: AudioSystem,
}

impl App {
    pub fn new(root_dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut state = AppState::new();

        // Initial library load
        let tracks = Self::load_tracks_from_directory(&root_dir)?;
        state.library.tracks = tracks;
        state.library.current_dir = root_dir;

        let shared_state = Arc::new(Mutex::new(state));
        Ok(Self {
            audio_system: AudioSystem::new(Arc::clone(&shared_state)),
            state: shared_state,
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
        let state = self.state.lock().unwrap();
        match state.playback.status {
            PlaybackStatus::Playing => {
                drop(state);
                self.audio_system.pause();
            }
            PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                let current_track = state.playback.current_track;
                let tracks_empty = state.library.tracks.is_empty();
                drop(state);

                if current_track.is_some() {
                    self.audio_system.resume();
                } else if !tracks_empty {
                    self.select_track(0)?;
                }
            }
        }
        Ok(())
    }

    pub fn select_track(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        let state = self.state.lock().unwrap();
        if index < state.library.tracks.len() {
            let current_track = index;
            drop(state);

            // Stop any current playback
            self.audio_system.stop();

            // Set the current track in state
            let mut state = self.state.lock().unwrap();
            state.set_current_track(current_track);
            drop(state);

            // Play the new track
            let _ = self.audio_system.play_track();
        }
        Ok(())
    }

    pub fn update(&mut self) {
        // TODO:  Update spectrum data for visualization

        let state = self.state.lock().unwrap();
        drop(state);

        // Update playback time
        self.audio_system.update_playback();

        // Sync audio controls
        //self.audio_system.update_audio_controls();
    }

    // Getter for UI to access state
    pub fn get_state(&self) -> Arc<Mutex<AppState>> {
        Arc::clone(&self.state)
    }
}

impl App {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool, Box<dyn Error>> {
        let mut state = self.state.lock().unwrap();

        match key_event.code {
            // Playback controls
            KeyCode::Enter => {
                if let Some(selected_index) = state.library.selected_index {
                    log::info!("Selected Index: {}", selected_index);
                    let index = selected_index;
                    drop(state);
                    self.select_track(index)?;
                }
            }
            KeyCode::Char('p') => {
                drop(state);
                self.toggle_playback()?;
            }
            // Navigation in track list
            KeyCode::Char('k') => {
                log::info!("K pressed -1");
                if let Some(current_index) = state.library.selected_index {
                    if current_index > 0 {
                        state.library.selected_index = Some(current_index - 1);
                    }
                }
            }
            KeyCode::Char('j') => {
                log::info!("J pressed +1");
                if let Some(current_index) = state.library.selected_index {
                    if current_index < state.library.tracks.len() - 1 {
                        state.library.selected_index = Some(current_index + 1);
                    }
                }
            }
            // Audio controls
            KeyCode::Left => {
                let (control, value) = if key_event.modifiers == KeyModifiers::SHIFT {
                    (AudioControl::Balance, state.audio.balance - 5.0)
                } else {
                    (AudioControl::Volume, state.audio.volume - 5.0)
                };
                state.set_audio_control(control, value);
            }
            KeyCode::Right => {
                let (control, value) = if key_event.modifiers == KeyModifiers::SHIFT {
                    (AudioControl::Balance, state.audio.balance + 5.0)
                } else {
                    (AudioControl::Volume, state.audio.volume + 5.0)
                };
                state.set_audio_control(control, value);
            }
            KeyCode::Up => {
                let (control, value) = if key_event.modifiers == KeyModifiers::SHIFT {
                    (AudioControl::Treble, state.audio.treble + 5.0)
                } else {
                    (AudioControl::Bass, state.audio.bass + 5.0)
                };
                state.set_audio_control(control, value);
            }
            KeyCode::Down => {
                let (control, value) = if key_event.modifiers == KeyModifiers::SHIFT {
                    (AudioControl::Treble, state.audio.treble - 5.0)
                } else {
                    (AudioControl::Bass, state.audio.bass - 5.0)
                };
                state.set_audio_control(control, value);
            }
            KeyCode::Char('q') => return Ok(false),
            _ => {}
        }
        Ok(true)
    }
}
