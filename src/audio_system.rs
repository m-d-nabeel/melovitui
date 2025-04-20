use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use rustfft::num_complex::Complex;
use rustfft::{FftDirection, FftPlanner};

use crate::controls::music_library::MusicLibrary;
use crate::controls::playback_control::{PlaybackControl, PlaybackStatus};
use crate::controls::spectrum::Spectrum;
use crate::{log_debug, log_error};

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
    spectrum: Arc<Mutex<Spectrum>>,
    visualizer_canvas: usize,
}
impl AudioSystem {
    pub fn new(
        library: Arc<Mutex<MusicLibrary>>,
        playback: Arc<Mutex<PlaybackControl>>,
    ) -> Result<Self, Box<dyn Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        let sound = Arc::new(Mutex::new(SoundControl::new()));
        let spectrum = Arc::new(Mutex::new(Spectrum::default()));

        log_debug!("Creating new AudioSystem instance");

        Ok(Self {
            library,
            playback,
            sound,
            sink,
            stream,
            stream_handle,
            spectrum,
            visualizer_canvas: 3,
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

        log_debug!("Track Path: {:?}", track_path);

        // For visualizer dumb approach
        // Try to compute FFT for visualization, but don't let it block playback
        {
            let mut spectrum = self.spectrum.lock();
            *spectrum = AudioSystem::fft(&track_path).unwrap_or_else(|e| {
                log_error!("Failed to compute FFT for visualization: {}", e);
                Spectrum::default()
            });
        }

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
                    log_debug!("Processing track: {:?}", track);
                    track.duration
                })
                .unwrap_or(Duration::ZERO);
            log_debug!("Track Duration: {:?}", duration);

            playback.start(index, duration);
        }

        // Apply current sound settings
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
    pub fn fft(path: impl AsRef<Path>) -> Result<Spectrum, Box<dyn Error>> {
        let src = File::open(path).unwrap();

        let source = Decoder::new_mp3(BufReader::new(src)).unwrap();

        let samples = source.convert_samples::<f32>();

        let ch = samples.channels() as usize;
        let rate = samples.sample_rate();
        assert!(rate % 60 == 0); // assume sample rate is divisible by 60 so that we can stream each frame 60 times per second
        let size = (rate / 60) as usize;
        let msize = size * ch;

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft(size, FftDirection::Forward);

        let mut slices = vec![];

        let hamming = apodize::hamming_iter(size)
            .map(|n| n as f32)
            .collect::<Vec<f32>>();

        let samples = samples.buffered();

        // TODO buffer not needed bc.: process(&mut slices[a..b])
        let mut buffer = vec![];
        for (k, b) in samples.enumerate() {
            if k != 0 && k % msize == 0 {
                fft.process(&mut buffer);
                if buffer.len() != size {
                    break;
                }
                slices.append(&mut buffer);
            }
            if k % ch == 0 {
                buffer.push(Complex {
                    re: b * hamming[(k % msize) / 2] as f32,
                    im: 0.0,
                });
            }
        }

        let out = slices
            .into_iter()
            .map(|v| (v.re * v.re + v.im * v.im).sqrt())
            .collect::<Vec<f32>>();

        // in buffer, frames are every `size`, 60 frames = 1sec

        Ok(Spectrum {
            inner: out,
            size,
            fps: 60,
        })
    }

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
    pub fn set_visualizer_canvas_type(&mut self, canvas_type: usize) {
        if canvas_type <= 9 {
            self.visualizer_canvas = canvas_type;
        }
    }
    /// Get a clone of the sound control state
    pub fn get_sound_state(&self) -> Arc<Mutex<SoundControl>> {
        Arc::clone(&self.sound)
    }

    pub fn get_visualizer_canvas_type(&self) -> usize {
        self.visualizer_canvas
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
        log_debug!("Volume adjusted to {}", self.volume);
    }

    /// Adjusts the bass by a delta and clamps it within the valid range
    pub fn adjust_bass(&mut self, delta: f32) {
        self.bass = (self.bass + delta).clamp(-100.0, 100.0);
        log_debug!("Bass adjusted to {}", self.bass);
    }

    /// Adjusts the treble by a delta and clamps it within the valid range
    pub fn adjust_treble(&mut self, delta: f32) {
        self.treble = (self.treble + delta).clamp(-100.0, 100.0);
        log_debug!("Treble adjusted to {}", self.treble);
    }

    /// Adjusts the balance by a delta and clamps it within the valid range
    pub fn adjust_balance(&mut self, delta: f32) {
        self.balance = (self.balance + delta).clamp(-100.0, 100.0);
        log_debug!("Balance adjusted to {}", self.balance);
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
