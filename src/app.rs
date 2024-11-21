use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::audio::AudioSystem;
use crate::controls::music_library::MusicLibrary;
use crate::controls::playback_control::{PlaybackControl, PlaybackStatus};
use crate::controls::sound_control::SoundControl;
use crate::controls::visualizer::Visualizer;

pub struct App {
    audio_system: Arc<Mutex<AudioSystem>>,
    library: Arc<Mutex<MusicLibrary>>,
    playback: Arc<Mutex<PlaybackControl>>,
    sound: Arc<Mutex<SoundControl>>,
    visualizer: Arc<Mutex<Visualizer>>,
}

impl App {
    pub fn new(root_dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        // Create initial components
        let library = Arc::new(Mutex::new(MusicLibrary::default()));
        let playback = Arc::new(Mutex::new(PlaybackControl::default()));
        let sound = Arc::new(Mutex::new(SoundControl::default()));
        let visualizer = Arc::new(Mutex::new(Visualizer::default()));

        // Load initial library
        {
            let mut lib = library.lock().unwrap();
            lib.load_tracks(root_dir)?;
        }

        // Initialize audio system with references to necessary components
        let audio_system = Arc::new(Mutex::new(AudioSystem::new(
            Arc::clone(&library),
            Arc::clone(&playback),
            Arc::clone(&sound),
            Arc::clone(&visualizer),
        )));

        Ok(Self {
            audio_system,
            library,
            playback,
            sound,
            visualizer,
        })
    }

    pub fn toggle_playback(&mut self) -> Result<(), Box<dyn Error>> {
        let playback = self.playback.lock().unwrap();
        match playback.status {
            PlaybackStatus::Playing => {
                drop(playback);
                self.audio_system.lock().unwrap().pause();
            }
            PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                let current_track = playback.current_track;
                let tracks_empty = self.library.lock().unwrap().tracks.is_empty();
                drop(playback);

                if current_track.is_some() {
                    self.audio_system.lock().unwrap().resume();
                } else if !tracks_empty {
                    self.select_track(0)?;
                }
            }
        }
        Ok(())
    }

    pub fn select_track(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        let library = self.library.lock().unwrap();
        if index < library.tracks.len() {
            drop(library);

            // Stop current playback
            self.audio_system.lock().unwrap().stop();

            // Update playback state
            {
                let mut playback = self.playback.lock().unwrap();
                playback.start(
                    index,
                    self.library.lock().unwrap().tracks[index]
                        .duration
                        .unwrap_or_default(),
                );
            }

            // Start playing new track
            self.audio_system.lock().unwrap().play_track(index)?;
        }
        Ok(())
    }

    pub fn update(&mut self) {
        // Update visualization
        if let Ok(_audio) = self.audio_system.lock() {
            if let Ok(visualizer) = self.visualizer.lock() {
                //visualizer.update_spectrum(audio.get_spectrum_data());
                log::debug!("VisualizerData: {:?}", visualizer);
            }
        }

        // Update playback state
        if let Ok(mut audio) = self.audio_system.lock() {
            audio.update_playback();
        }
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
        Arc::clone(&self.sound)
    }
    pub fn get_playback_state(&self) -> Arc<Mutex<PlaybackControl>> {
        Arc::clone(&self.playback)
    }
}

impl App {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool, Box<dyn Error>> {
        match key_event.code {
            KeyCode::Enter => {
                let selected_index = match self.library.lock() {
                    Ok(library) => {
                        let index = library.selected_index;
                        drop(library);
                        index
                    }
                    Err(err) => {
                        log::error!("Failed to lock library on Enter key: {}", err);
                        None
                    }
                };

                if let Some(index) = selected_index {
                    match self.select_track(index) {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Error selecting track {}: {}", index, err);
                        }
                    }
                } else {
                    log::debug!("No track selected on Enter key");
                }
            }
            KeyCode::Char('p') => match self.toggle_playback() {
                Ok(_) => log::debug!("Playback toggled"),
                Err(err) => log::error!("Error toggling playback: {}", err),
            },
            KeyCode::Char('k') => match self.library.lock() {
                Ok(mut library) => {
                    library.select_previous();
                    log::debug!("Selected previous track");
                }
                Err(err) => {
                    log::error!(
                        "Failed to lock library for previous track selection: {}",
                        err
                    );
                }
            },
            KeyCode::Char('j') => match self.library.lock() {
                Ok(mut library) => {
                    library.select_next();
                    log::debug!("Selected next track");
                }
                Err(err) => {
                    log::error!("Failed to lock library for next track selection: {}", err);
                }
            },
            // Audio controls
            KeyCode::Left | KeyCode::Right => match self.sound.lock() {
                Ok(mut sound) => {
                    let delta = if key_event.code == KeyCode::Left {
                        -5.0
                    } else {
                        5.0
                    };

                    if key_event.modifiers == KeyModifiers::SHIFT {
                        let new_balance = sound.balance + delta;
                        sound.set_balance(new_balance);
                        log::debug!("Balance adjusted to {}", new_balance);
                    } else {
                        let new_volume = sound.volume + delta;
                        sound.set_volume(new_volume);
                        log::debug!("Volume adjusted to {}", new_volume);
                    }
                }
                Err(err) => {
                    log::error!("Failed to lock sound for left/right adjustment: {}", err);
                }
            },
            KeyCode::Up | KeyCode::Down => match self.sound.lock() {
                Ok(mut sound) => {
                    let delta = if key_event.code == KeyCode::Down {
                        -5.0
                    } else {
                        5.0
                    };

                    if key_event.modifiers == KeyModifiers::SHIFT {
                        let new_treble = sound.treble + delta;
                        sound.set_treble(new_treble);
                        log::debug!("Treble adjusted to {}", new_treble);
                    } else {
                        let new_bass = sound.bass + delta;
                        sound.set_bass(new_bass);
                        log::debug!("Bass adjusted to {}", new_bass);
                    }
                }
                Err(err) => {
                    log::error!("Failed to lock sound for up/down adjustment: {}", err);
                }
            },
            KeyCode::Char('q') => {
                log::info!("Quit key pressed");
                return Ok(false);
            }
            _ => {
                log::debug!("Unhandled key event: {:?}", key_event);
            }
        }
        Ok(true)
    }
}
