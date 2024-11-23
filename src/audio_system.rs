use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::controls::music_library::MusicLibrary;
use crate::controls::playback_control::{PlaybackControl, PlaybackStatus};
use crate::controls::visualizer::Visualizer;

/// Represents audio settings with well-defined constraints
#[derive(Debug, Clone)]
pub struct SoundControl {
    volume: f32,
    bass: f32,
    treble: f32,
    balance: f32,
}

impl Default for SoundControl {
    fn default() -> Self {
        Self {
            volume: 50.0,
            bass: 0.0,
            treble: 0.0,
            balance: 0.0,
        }
    }
}

impl SoundControl {
    /// Creates a new SoundControl with validated initial values
    pub fn new() -> Self {
        Self::default()
    }

    /// Adjusts the volume by a delta and clamps it within the valid range
    pub fn adjust_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 100.0);
        log::info!("Volume adjusted to {}", self.volume);
    }

    /// Adjusts the bass by a delta and clamps it within the valid range
    pub fn adjust_bass(&mut self, delta: f32) {
        self.bass = (self.bass + delta).clamp(-100.0, 100.0);
        log::info!("Bass adjusted to {}", self.bass);
    }

    /// Adjusts the treble by a delta and clamps it within the valid range
    pub fn adjust_treble(&mut self, delta: f32) {
        self.treble = (self.treble + delta).clamp(-100.0, 100.0);
        log::info!("Treble adjusted to {}", self.treble);
    }

    /// Adjusts the balance by a delta and clamps it within the valid range
    pub fn adjust_balance(&mut self, delta: f32) {
        self.balance = (self.balance + delta).clamp(-100.0, 100.0);
        log::info!("Balance adjusted to {}", self.balance);
    }
    /// Getter for volume
    pub fn volume(&self) -> f32 {
        self.volume
    }
    /// Getter for bass
    pub fn bass(&self) -> f32 {
        self.bass
    }

    /// Getter for treble
    pub fn treble(&self) -> f32 {
        self.treble
    }

    /// Getter for balance
    pub fn balance(&self) -> f32 {
        self.balance
    }
}

/// Primary audio system managing playback, library, and sound controls
pub struct AudioSystem {
    library: Arc<Mutex<MusicLibrary>>,
    playback: Arc<Mutex<PlaybackControl>>,
    sound: Arc<Mutex<SoundControl>>,
    sink: Sink,
    #[allow(dead_code)]
    stream: OutputStream,
    #[allow(dead_code)]
    stream_handle: OutputStreamHandle,
    visualizer: Arc<Mutex<Visualizer>>,
}
impl AudioSystem {
    pub fn new(
        library: Arc<Mutex<MusicLibrary>>,
        playback: Arc<Mutex<PlaybackControl>>,
        visualizer: Arc<Mutex<Visualizer>>,
    ) -> Result<Self, Box<dyn Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        let sound = Arc::new(Mutex::new(SoundControl::new()));

        Ok(Self {
            library,
            playback,
            sound,
            sink,
            stream,
            stream_handle,
            visualizer,
        })
    }
}

impl AudioSystem {
    /// Play a track by index with comprehensive error handling
    pub fn play_track(&mut self, track_index: Option<usize>) -> Result<(), Box<dyn Error>> {
        let index: usize;
        if let Some(idx) = track_index {
            index = idx;
        } else {
            index = self.library.lock().selected_index.unwrap_or(0);
        }

        let track_path = {
            let library = self.library.lock();
            library
                .tracks
                .get(index)
                .ok_or_else(|| format!("Invalid track index: {}", index))?
                .path
                .clone()
        };

        log::info!("Track Path: {:?}", track_path);

        // Decode and play the track
        let file = std::fs::File::open(&track_path)?;
        let source = rodio::Decoder::new(file)?;

        self.sink.clear();
        self.sink.append(source);
        self.sink.play();

        // Update playback state
        {
            let mut playback = self.playback.lock();
            let duration = self
                .library
                .lock()
                .tracks
                .get(index)
                .and_then(|track| {
                    log::info!("==>Track: {:?}", track);
                    track.duration
                })
                .unwrap_or(Duration::ZERO);
            log::info!("==>Track Duration: {:?}", duration);

            playback.start(index, duration);
        }

        // Apply current sound settings
        self.apply_sound_settings();

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
                    log::info!("==>ElapsedTime Update Failed")
                }
            }
        }

        // Advance track if needed
        if should_advance {
            self.advance_track();
        }

        // Update visualizer with dummy spectrum (placeholder)
        let mut visualizer = self.visualizer.lock();
        let dummy_spectrum = vec![0.0; 32];
        visualizer.update_spectrum(dummy_spectrum);
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

            if let Err(e) = self.play_track(Some(next_index)) {
                log::error!("Failed to advance track: {}", e);
            }
        }
    }

    /// Apply current sound settings to audio output
    fn apply_sound_settings(&mut self) {
        let sound = self.sound.lock();
        let volume = sound.volume() / 100.0;
        self.sink.set_volume(volume);

        // TODO: Implement advanced audio processing
        // - Bass and treble adjustments
        // - Balance control
        drop(sound);
    }

    /// Toggle playback state intelligently
    pub fn toggle_playback(&mut self) -> Result<(), Box<dyn Error>> {
        let playback = self.playback.lock();
        match playback.status {
            PlaybackStatus::Playing => {
                drop(playback);
                self.pause();
                log::info!("Playback paused");
            }
            PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                let current_track = playback.current_track;
                let tracks_empty = self.library.lock().tracks.is_empty();
                drop(playback);

                match (current_track, tracks_empty) {
                    (Some(_), _) => {
                        self.resume();
                        log::info!("Playback resumed");
                    }
                    (None, false) => {
                        self.play_track(Some(0))?;
                        log::info!("Started first track");
                    }
                    _ => log::warn!("No tracks available"),
                }
            }
        }
        Ok(())
    }
}

impl AudioSystem {
    /// Pause current playback
    pub fn pause(&mut self) {
        let mut playback = self.playback.lock();
        playback.status = PlaybackStatus::Paused;
        self.sink.pause();
    }

    /// Resume paused playback
    pub fn resume(&mut self) {
        let mut playback = self.playback.lock();
        playback.status = PlaybackStatus::Playing;
        self.sink.play();
    }

    /// Stop current playback
    pub fn stop(&mut self) {
        let mut playback = self.playback.lock();
        playback.status = PlaybackStatus::Stopped;
        playback.elapsed = Duration::ZERO;
        self.sink.stop();
    }
}
impl AudioSystem {
    pub fn adjust_volume(&mut self, delta: f32) {
        {
            let mut sound_control = self.sound.lock();
            sound_control.adjust_volume(delta);
        }
        self.apply_sound_settings();
    }

    pub fn adjust_bass(&mut self, delta: f32) {
        {
            let mut sound_control = self.sound.lock();
            sound_control.adjust_bass(delta);
        }
        self.apply_sound_settings();
    }

    pub fn adjust_treble(&mut self, delta: f32) {
        {
            let mut sound_control = self.sound.lock();
            sound_control.adjust_treble(delta);
        }
        self.apply_sound_settings();
    }

    pub fn adjust_balance(&mut self, delta: f32) {
        {
            let mut sound_control = self.sound.lock();
            sound_control.adjust_balance(delta);
        }
        self.apply_sound_settings();
    }
    /// Get a clone of the sound control state
    pub fn get_sound_state(&self) -> Arc<Mutex<SoundControl>> {
        Arc::clone(&self.sound)
    }
}
