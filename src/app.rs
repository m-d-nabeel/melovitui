use parking_lot::Mutex;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use crate::audio_system::{AudioSystem, SoundControl};
use crate::controls::music_library::MusicLibrary;
use crate::controls::playback_control::PlaybackControl;
use crate::controls::visualizer::Visualizer;

pub struct App {
    audio_system: Arc<Mutex<AudioSystem>>,
    library: Arc<Mutex<MusicLibrary>>,
    playback: Arc<Mutex<PlaybackControl>>,
    visualizer: Arc<Mutex<Visualizer>>,
}

impl App {
    pub fn new(root_dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        // Create initial components
        let library = MusicLibrary::new(root_dir)?;
        let library = Arc::new(Mutex::new(library));
        let playback = Arc::new(Mutex::new(PlaybackControl::default()));
        let visualizer = Arc::new(Mutex::new(Visualizer::default()));

        // Initialize audio system with references to necessary components
        // [[CHECKPOINT]]
        let audio_system = AudioSystem::new(
            Arc::clone(&library),
            Arc::clone(&playback),
            Arc::clone(&visualizer),
        )?;

        #[allow(clippy::arc_with_non_send_sync)]
        let audio_system = Arc::new(Mutex::new(audio_system));

        Ok(Self {
            audio_system,
            library,
            playback,
            visualizer,
        })
    }

    pub fn update(&mut self) {
        let mut audio = self.audio_system.lock();
        // Update playback state
        audio.update_playback();
        // TODO: Update visualization
    }
}

impl App {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool, Box<dyn Error>> {
        match key_event.code {
            KeyCode::Enter => {
                let current_index = self.library.lock().current_index;
                self.library.lock().select_track(current_index);
                self.playback.lock().current_track = Some(current_index);
                self.audio_system.lock().play_track(None)?;
            }
            KeyCode::Char('p') => {
                if let Err(err) = self.audio_system.lock().toggle_playback() {
                    log::error!("Error toggling playback: {}", err);
                }
            }
            KeyCode::Char('k') => {
                self.library.lock().select_previous();
                log::info!("Selected previous track");
            }
            KeyCode::Char('j') => {
                self.library.lock().select_next();
                log::info!("Selected next track");
            }

            KeyCode::Left | KeyCode::Right => {
                let delta = if key_event.code == KeyCode::Left {
                    -5.0
                } else {
                    5.0
                };

                if key_event.modifiers == KeyModifiers::SHIFT {
                    self.audio_system.lock().adjust_balance(delta);
                } else {
                    self.audio_system.lock().adjust_volume(delta);
                }
            }

            KeyCode::Up | KeyCode::Down => {
                let delta = if key_event.code == KeyCode::Down {
                    -5.0
                } else {
                    5.0
                };

                if key_event.modifiers == KeyModifiers::SHIFT {
                    self.audio_system.lock().adjust_treble(delta);
                } else {
                    self.audio_system.lock().adjust_bass(delta);
                }
            }
            KeyCode::Char('s') => {
                self.audio_system.lock().stop();
            }
            KeyCode::Char('q') => {
                log::info!("Quit key pressed");
                return Ok(false);
            }
            _ => {
                log::info!("Unhandled key event: {:?}", key_event);
            }
        }
        Ok(true)
    }
}

impl App {
    pub fn get_library_state(&self) -> Arc<Mutex<MusicLibrary>> {
        Arc::clone(&self.library)
    }
    pub fn get_visualizer_state(&self) -> Arc<Mutex<Visualizer>> {
        Arc::clone(&self.visualizer)
    }
    pub fn get_sound_state(&self) -> Arc<Mutex<SoundControl>> {
        Arc::clone(&self.audio_system.lock().get_sound_state())
    }
    pub fn get_playback_state(&self) -> Arc<Mutex<PlaybackControl>> {
        Arc::clone(&self.playback)
    }
}
