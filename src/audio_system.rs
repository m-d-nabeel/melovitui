use std::error::Error;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;

use crate::controls::audio_engine::AudioEngine;
use crate::controls::music_library::MusicLibrary;
use crate::controls::playback_state::{PlaybackState, PlaybackStatus};
use crate::controls::sound_control::SoundControl;
use crate::controls::spectrum::Spectrum;
use crate::{log_debug, log_error};

/// Primary audio system that manages playback_state, sound_control processing, music library,
/// and visualization.
///
/// The `AudioSystem` is the central component responsible for:
/// - Playing and controlling audio playback_state
/// - Applying audio effects and adjustments (volume, bass, treble, pitch)
/// - Track selection and progression
/// - Audio visualization data processing
pub struct AudioSystem {
    library: Arc<Mutex<MusicLibrary>>,
    playback_state: Arc<Mutex<PlaybackState>>,
    sound_control: Arc<Mutex<SoundControl>>,
    audio_engine: Rc<Mutex<AudioEngine>>,
    spectrum: Arc<Mutex<Spectrum>>,
    visualizer_canvas: usize,
}
impl AudioSystem {
    pub fn new(
        library: Arc<Mutex<MusicLibrary>>,
        playback_state: Arc<Mutex<PlaybackState>>,
    ) -> Result<Self, Box<dyn Error>> {
        let sound_control = Arc::new(Mutex::new(SoundControl::new()));
        let spectrum = Arc::new(Mutex::new(Spectrum::default()));
        let audio_engine = Rc::new(Mutex::new(AudioEngine::new().unwrap()));

        log_debug!("Creating new AudioSystem instance");

        Ok(Self {
            library,
            playback_state,
            sound_control,
            spectrum,
            audio_engine,
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

        {
            let mut spectrum = self.spectrum.lock();
            *spectrum = Spectrum::fft_async(&track_path)
        }

        match self.audio_engine.lock().play(&track_path) {
            Ok(_) => log_debug!("Now playing: {:?}", track_path),
            Err(e) => log_error!("Failed to play {:?}: {:?}", track_path, e),
        }
        // Update playback_state state
        {
            let mut playback_state = self.playback_state.lock();
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

            playback_state.start(index, duration);
        }

        // Apply current sound_control settings
        self.apply_sound_settings();

        log_debug!("Track playback_state set to playing successfully");
        Ok(())
    }

    /// Update playback_state progress and handle track completion
    pub fn update_playback(&mut self) {
        if self.playback_state.lock().status != PlaybackStatus::Playing {
            return;
        }
        self.spectrum.lock().update();
        let audio_engine = self.audio_engine.lock();
        if audio_engine.is_sink_empty() {
            drop(audio_engine);
            self.advance_track();
            return;
        }

        let raw_elapsed = audio_engine.get_current_pos();
        let speed_factor = self.get_current_speed();
        let adjusted_elapsed = if speed_factor != 0.0 {
            Duration::from_secs_f64(raw_elapsed.as_secs_f64() * speed_factor as f64)
        } else {
            raw_elapsed
        };

        self.playback_state.lock().update_elapsed(adjusted_elapsed);
    }

    /// Advance to the next track automatically
    fn advance_track(&mut self) {
        let mut library = self.library.lock();
        let current_track = {
            let playback_state = self.playback_state.lock();
            playback_state.current_track
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
        self.audio_engine.lock().apply_effects(&sound_control);
    }

    /// Toggle playback_state between play and pause
    pub fn toggle_playback(&mut self) -> Result<(), Box<dyn Error>> {
        let current_status = {
            let playback_state = self.playback_state.lock();
            playback_state.status.clone()
        };

        match current_status {
            PlaybackStatus::Playing => {
                self.pause();
                {
                    let mut playback_state = self.playback_state.lock();
                    playback_state.status = PlaybackStatus::Paused;
                }
                log_debug!("playback_state paused");
            }
            PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                let (current_track, library_is_empty) = {
                    let playback_state = self.playback_state.lock();
                    let library = self.library.lock();
                    (playback_state.current_track, library.tracks.is_empty())
                };

                match (current_track, library_is_empty) {
                    (Some(_), _) => {
                        self.resume();
                        {
                            let mut playback_state = self.playback_state.lock();
                            playback_state.status = PlaybackStatus::Playing;
                        }
                        log_debug!("playback_state resumed");
                    }
                    (None, false) => {
                        self.play_track(Some(0))?;
                        log_debug!("Started first track");
                    }
                    _ => {
                        log_error!("No tracks available");
                        log_error!("Cannot start playback_state - library is empty");
                    }
                }
            }
        }

        Ok(())
    }
}

impl AudioSystem {
    pub fn get_current_frame(&self) -> Vec<f32> {
        let spectrum = self.spectrum.lock();
        if spectrum.processing || spectrum.size == 0 || spectrum.inner.is_empty() {
            return vec![];
        }

        let elapsed = self.playback_state.lock().elapsed.as_millis() as usize;

        // Calculate the pointer offset for the current frame
        let ptr = spectrum.size * (elapsed as f32 / (1000.0 / spectrum.fps as f32)) as usize;

        // Ensure bounds safety
        if ptr + spectrum.size > spectrum.inner.len() {
            return vec![];
        }

        // Copy the current frame into a new vector
        spectrum.inner[ptr..ptr + spectrum.size].to_vec()
    }
}

impl AudioSystem {
    /// Pause current playback_state
    pub fn pause(&mut self) {
        let mut playback_state = self.playback_state.lock();
        playback_state.status = PlaybackStatus::Paused;
        self.audio_engine.lock().pause();
    }

    /// Resume paused playback_state
    pub fn resume(&mut self) {
        let mut playback_state = self.playback_state.lock();
        playback_state.status = PlaybackStatus::Playing;
        self.audio_engine.lock().resume();
    }

    /// Stop current playback_state
    pub fn stop(&mut self) {
        let mut playback_state = self.playback_state.lock();
        playback_state.status = PlaybackStatus::Stopped;
        playback_state.elapsed = Duration::ZERO;
        self.audio_engine.lock().stop();
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

    pub fn adjust_pitch(&mut self, delta: f32) {
        {
            let mut sound_control = self.sound_control.lock();
            sound_control.adjust_pitch(delta);
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

        // Calculate actual audio position
        let audio_engine = self.audio_engine.lock();
        let actual_position = audio_engine.get_current_pos();
        drop(audio_engine);

        // Add the fixed seek value to the actual audio position
        let new_audio_position = actual_position + Duration::from_secs_f32(seek_value);

        match self.audio_engine.lock().seek_control(new_audio_position) {
            Ok(_) => {
                // Update the elapsed time based on the current speed factor
                let speed_factor = self.get_current_speed();
                let new_playback_position =
                    Duration::from_secs_f64(new_audio_position.as_secs_f64() * speed_factor as f64);
                self.playback_state
                    .lock()
                    .update_elapsed(new_playback_position);
                log_debug!("Successfully sought forward to {:?}", new_playback_position);
            }
            Err(e) => log_error!("Failed to seek forward: {}", e),
        };
    }

    pub fn seek_backward(&mut self, delta: Option<f32>) {
        let seek_value = delta.unwrap_or(10.0);

        // Calculate actual audio position
        let audio_engine = self.audio_engine.lock();
        let actual_position = audio_engine.get_current_pos();
        drop(audio_engine);

        // Subtract the fixed seek value from the actual audio position
        let new_audio_position =
            actual_position.saturating_sub(Duration::from_secs_f32(seek_value));

        match self.audio_engine.lock().seek_control(new_audio_position) {
            Ok(_) => {
                // Update the elapsed time based on the current speed factor
                let speed_factor = self.get_current_speed();
                let new_playback_position =
                    Duration::from_secs_f64(new_audio_position.as_secs_f64() * speed_factor as f64);
                self.playback_state
                    .lock()
                    .update_elapsed(new_playback_position);
                log_debug!(
                    "Successfully sought backward to {:?}",
                    new_playback_position
                );
            }
            Err(e) => log_error!("Failed to seek backward: {}", e),
        };
    }
    pub fn get_current_speed(&self) -> f32 {
        let pitch = self.sound_control.lock().pitch();
        AudioEngine::calc_playback_speed(pitch)
    }
}
