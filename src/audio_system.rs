use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;

use crate::controls::music_library::MusicLibrary;
use crate::controls::playback_control::{PlaybackControl, PlaybackStatus};
use crate::controls::playback_engine::PlaybackEngine;
use crate::controls::sound_control::SoundControl;
use crate::controls::spectrum::Spectrum;
use crate::{log_debug, log_error};

/// Primary audio system that manages playback, sound_control processing, music library,
/// and visualization.
///
/// The `AudioSystem` is the central component responsible for:
/// - Playing and controlling audio playback
/// - Applying audio effects and adjustments (volume, bass, treble, balance)
/// - Track selection and progression
/// - Audio visualization data processing
pub struct AudioSystem {
    library: Arc<Mutex<MusicLibrary>>,
    playback: Arc<Mutex<PlaybackControl>>,
    sound_control: Arc<Mutex<SoundControl>>,
    playback_engine: Arc<Mutex<PlaybackEngine>>,
    spectrum: Arc<Mutex<Spectrum>>,
    visualizer_canvas: usize,
}
impl AudioSystem {
    pub fn new(
        library: Arc<Mutex<MusicLibrary>>,
        playback: Arc<Mutex<PlaybackControl>>,
    ) -> Result<Self, Box<dyn Error>> {
        let sound_control = Arc::new(Mutex::new(SoundControl::new()));
        let spectrum = Arc::new(Mutex::new(Spectrum::default()));
        let playback_engine = Arc::new(Mutex::new(PlaybackEngine::new().unwrap()));

        log_debug!("Creating new AudioSystem instance");

        Ok(Self {
            library,
            playback,
            sound_control,
            spectrum,
            playback_engine,
            visualizer_canvas: 0,
        })
    }
}

impl AudioSystem {
    /// Play a track by index
    pub fn play_track(&mut self, track_index: Option<usize>) -> Result<(), Box<dyn Error>> {
        let index = match track_index {
            Some(idx) => idx,
            None => self
                .library
                .lock()
                .selected_index
                .expect("music library must be empty"),
        };

        let track_path = {
            let library = self.library.lock();
            library
                .tracks
                .get(index)
                .ok_or_else(|| format!("Invalid track index: {}", index))?
                .path
                .clone()
        };

        // For visualizer dumb approach
        // Try to compute FFT for visualization, but don't let it block playback
        // needs concurrency that will need communication channel to threads
        {
            let mut spectrum = self.spectrum.lock();
            *spectrum = Spectrum::fft(&track_path).unwrap_or_else(|e| {
                log_error!("Failed to compute FFT for visualization: {}", e);
                Spectrum::default()
            });
        }

        match self.playback_engine.lock().play(&track_path) {
            Ok(_) => log_debug!("Now playing: {:?}", track_path),
            Err(e) => log_error!("Failed to play {:?}: {:?}", track_path, e),
        }
        // Update playback state
        {
            let mut playback = self.playback.lock();
            let duration = self
                .library
                .lock()
                .tracks
                .get(index)
                .and_then(|track| {
                    log_debug!("Processing track: {:?}", track);
                    track.duration
                })
                .unwrap_or(Duration::ZERO);
            log_debug!("Track Duration: {:?}", duration);

            playback.start(index, duration);
        }

        // Apply current sound_control settings
        self.apply_sound_settings();

        log_debug!("Track playback started successfully");
        Ok(())
    }

    /// Update playback progress and handle track completion
    pub fn update_playback(&mut self) {
        let mut should_advance = false;

        // Check playback status and update elapsed time
        {
            let mut playback = self.playback.lock();
            if playback.status == PlaybackStatus::Playing {
                if let Some(current_track_index) = playback.current_track {
                    let track_duration = self
                        .library
                        .lock()
                        .tracks
                        .get(current_track_index)
                        .and_then(|track| track.duration)
                        .unwrap_or(Duration::ZERO);

                    // Increment elapsed time
                    let current_elapsed = playback.elapsed + Duration::from_millis(16);
                    playback.update_elapsed(current_elapsed);

                    // Check if track has finished
                    if current_elapsed >= track_duration {
                        should_advance = true;
                    }
                } else {
                    log_error!("Failed to update elapsed time: no current track");
                }
            }
        }

        // Advance track if needed
        if should_advance {
            self.advance_track();
        }
    }

    /// Advance to the next track automatically
    fn advance_track(&mut self) {
        let mut library = self.library.lock();
        let current_track = {
            let playback = self.playback.lock();
            playback.current_track
        };

        if let Some(current_index) = current_track {
            let next_index = (current_index + 1) % library.tracks.len();
            library.selected_index = Some(next_index);
            drop(library);

            match self.play_track(Some(next_index)) {
                Ok(_) => {
                    log_debug!("Successfully advanced to next track");
                }
                Err(e) => {
                    log_error!("Failed to advance to next track: {}", e);
                }
            }
        } else {
            log_error!("Cannot advance track: no current track");
        }
    }

    /// Apply current sound_control settings to audio output
    fn apply_sound_settings(&mut self) {
        let sound_control = self.sound_control.lock();
        self.playback_engine.lock().apply_effects(&sound_control);
    }

    /// Toggle playback between play and pause
    pub fn toggle_playback(&mut self) -> Result<(), Box<dyn Error>> {
        let current_status = {
            let playback = self.playback.lock();
            playback.status.clone()
        };

        match current_status {
            PlaybackStatus::Playing => {
                self.pause();
                {
                    let mut playback = self.playback.lock();
                    playback.status = PlaybackStatus::Paused;
                }
                log_debug!("Playback paused");
            }
            PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                let (current_track, library_is_empty) = {
                    let playback = self.playback.lock();
                    let library = self.library.lock();
                    (playback.current_track, library.tracks.is_empty())
                };

                match (current_track, library_is_empty) {
                    (Some(_), _) => {
                        self.resume();
                        {
                            let mut playback = self.playback.lock();
                            playback.status = PlaybackStatus::Playing;
                        }
                        log_debug!("Playback resumed");
                    }
                    (None, false) => {
                        self.play_track(Some(0))?;
                        log_debug!("Started first track");
                    }
                    _ => {
                        log_error!("No tracks available");
                        log_error!("Cannot start playback - library is empty");
                    }
                }
            }
        }

        Ok(())
    }
}

impl AudioSystem {
    pub fn get_current_frame(&self) -> Vec<f32> {
        // Lock the spectrum to gain access
        let spectrum = self.spectrum.lock();

        let elapsed = self.playback.lock().elapsed.as_millis() as usize;

        // Calculate the pointer offset for the current frame
        let ptr = spectrum.size * (elapsed as f32 / (1000.0 / spectrum.fps as f32)) as usize;

        // Ensure bounds safety
        if ptr + spectrum.size > spectrum.inner.len() {
            return vec![]; // Return an empty vector if out of bounds
        }

        // Copy the current frame into a new vector
        spectrum.inner[ptr..ptr + spectrum.size].to_vec()
    }
}

impl AudioSystem {
    /// Pause current playback
    pub fn pause(&mut self) {
        let mut playback = self.playback.lock();
        playback.status = PlaybackStatus::Paused;
        self.playback_engine.lock().pause();
    }

    /// Resume paused playback
    pub fn resume(&mut self) {
        let mut playback = self.playback.lock();
        playback.status = PlaybackStatus::Playing;
        self.playback_engine.lock().resume();
    }

    /// Stop current playback
    pub fn stop(&mut self) {
        let mut playback = self.playback.lock();
        playback.status = PlaybackStatus::Stopped;
        playback.elapsed = Duration::ZERO;
        self.playback_engine.lock().stop();
    }
}

impl AudioSystem {
    pub fn adjust_volume(&mut self, delta: f32) {
        {
            let mut sound_control = self.sound_control.lock();
            sound_control.adjust_volume(delta);
        }
        self.apply_sound_settings();
    }

    pub fn adjust_bass(&mut self, delta: f32) {
        {
            let mut sound_control = self.sound_control.lock();
            sound_control.adjust_bass(delta);
        }
        self.apply_sound_settings();
    }

    pub fn adjust_treble(&mut self, delta: f32) {
        {
            let mut sound_control = self.sound_control.lock();
            sound_control.adjust_treble(delta);
        }
        self.apply_sound_settings();
    }

    pub fn adjust_balance(&mut self, delta: f32) {
        {
            let mut sound_control = self.sound_control.lock();
            sound_control.adjust_balance(delta);
        }
        self.apply_sound_settings();
    }
    pub fn set_visualizer_canvas_type(&mut self, canvas_type: usize) {
        if canvas_type <= 9 {
            self.visualizer_canvas = canvas_type;
        }
    }
    /// Get a clone of the sound_control control state
    pub fn get_sound_state(&self) -> Arc<Mutex<SoundControl>> {
        Arc::clone(&self.sound_control)
    }

    pub fn get_visualizer_canvas_type(&self) -> usize {
        self.visualizer_canvas
    }
}

impl AudioSystem {
    pub fn seek_forward(&mut self, delta: Option<f32>) {
        let seek_value = delta.unwrap_or(10.0);
        let current = self.playback.lock().elapsed;
        let new_position = current + Duration::from_secs_f32(seek_value);

        match self.playback_engine.lock().seek_control(new_position) {
            Ok(_) => self.playback.lock().update_elapsed(new_position),
            Err(e) => log_error!("Failed to seek forward: {}", e),
        };
    }

    pub fn seek_backward(&mut self, delta: Option<f32>) {
        let seek_value = delta.unwrap_or(10.0);
        let current = self.playback.lock().elapsed;
        let new_position = current.saturating_sub(Duration::from_secs_f32(seek_value));
        match self.playback_engine.lock().seek_control(new_position) {
            Ok(_) => self.playback.lock().update_elapsed(new_position),
            Err(e) => log_error!("Failed to seek backward: {}", e),
        }
    }
}
