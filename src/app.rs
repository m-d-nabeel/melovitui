use parking_lot::Mutex;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use crate::audio_system::AudioSystem;
use crate::controls::keybindings::Keybindings;
use crate::controls::music_library::MusicLibrary;
use crate::controls::playback_state::PlaybackState;
use crate::controls::sound_control::SoundControl;
use crate::{log_debug, log_error};

/// Main application state container and controller.
///
/// The `App` struct is the central coordinator of the application, managing:
/// - Audio system and playback
/// - Music library browsing
/// - User input handling via keybindings
/// - UI state (help overlay visibility)
pub struct App {
    audio_system: Arc<Mutex<AudioSystem>>,
    library: Arc<Mutex<MusicLibrary>>,
    playback: Arc<Mutex<PlaybackState>>,
    keybindings: Keybindings,
    pub show_help: bool,
}

impl App {
    pub fn new(root_dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        // Create initial components
        let library = MusicLibrary::new(root_dir)?;
        let library = Arc::new(Mutex::new(library));
        let playback = Arc::new(Mutex::new(PlaybackState::default()));

        // Initialize audio system with references to necessary components
        // [[CHECKPOINT]]
        let audio_system = AudioSystem::new(Arc::clone(&library), Arc::clone(&playback))?;

        #[allow(clippy::arc_with_non_send_sync)]
        let audio_system = Arc::new(Mutex::new(audio_system));

        Ok(Self {
            audio_system,
            library,
            playback,
            keybindings: Keybindings::new(),
            show_help: false,
        })
    }

    pub fn update(&mut self) {
        let mut audio = self.audio_system.lock();
        // Update playback state update visualizer with it
        audio.update_playback();
    }
}

impl App {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool, Box<dyn Error>> {
        // Check if the '?' key was pressed to toggle help
        if key_event.code == KeyCode::Char('?') {
            self.show_help = !self.show_help;
            return Ok(true);
        }

        // If help is showing, pressing any key dismisses it (except '?' which already toggles it)
        if self.show_help {
            self.show_help = false;
            return Ok(true);
        }

        // Check if we have a keybinding for this key
        if let Some(action) = self.keybindings.get_action(&key_event) {
            match action.name.as_str() {
                "play_selected" => {
                    let current_index = self.library.lock().current_index;
                    self.library.lock().select_track(current_index);
                    self.playback.lock().current_track = Some(current_index);
                    self.audio_system.lock().play_track(None)?;
                }
                "toggle_playback" => {
                    if let Err(err) = self.audio_system.lock().toggle_playback() {
                        log_error!("Error toggling playback: {}", err);
                    }
                }
                "select_previous" => {
                    self.library.lock().select_previous();
                    log_debug!("Selected previous track");
                }
                "select_next" => {
                    self.library.lock().select_next();
                    log_debug!("Selected next track");
                }
                "seek_forward" => {
                    self.audio_system.lock().seek_forward(None);
                }
                "seek_backward" => {
                    self.audio_system.lock().seek_backward(None);
                }
                "volume_down" => {
                    self.audio_system.lock().adjust_volume(-5.0);
                }
                "volume_up" => {
                    self.audio_system.lock().adjust_volume(5.0);
                }
                "balance_left" => {
                    self.audio_system.lock().adjust_balance(-5.0);
                }
                "balance_right" => {
                    self.audio_system.lock().adjust_balance(5.0);
                }
                "bass_up" => {
                    self.audio_system.lock().adjust_bass(5.0);
                }
                "bass_down" => {
                    self.audio_system.lock().adjust_bass(-5.0);
                }
                "treble_up" => {
                    self.audio_system.lock().adjust_treble(5.0);
                }
                "treble_down" => {
                    self.audio_system.lock().adjust_treble(-5.0);
                }
                "stop" => {
                    self.audio_system.lock().stop();
                }
                "quit" => {
                    log_debug!("Quit key pressed");
                    return Ok(false);
                }
                name if name.starts_with("visualizer_mode_") => {
                    if let Some(canvas_type) = name.chars().last().and_then(|c| c.to_digit(10)) {
                        self.audio_system
                            .lock()
                            .set_visualizer_canvas_type(canvas_type as usize);
                    }
                }
                _ => {
                    log_debug!("Unhandled action: {}", action.name);
                }
            }
            return Ok(true);
        }

        log_debug!("Unhandled key event: {:?}", key_event);
        Ok(true)
    }
}

impl App {
    pub fn get_library_state(&self) -> Arc<Mutex<MusicLibrary>> {
        Arc::clone(&self.library)
    }

    pub fn get_sound_state(&self) -> Arc<Mutex<SoundControl>> {
        Arc::clone(&self.audio_system.lock().get_sound_state())
    }

    pub fn get_playback_state(&self) -> Arc<Mutex<PlaybackState>> {
        Arc::clone(&self.playback)
    }

    pub fn get_audio_system(&self) -> Arc<Mutex<AudioSystem>> {
        Arc::clone(&self.audio_system)
    }

    pub fn get_keybindings(&self) -> &Keybindings {
        &self.keybindings
    }
}
