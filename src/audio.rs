use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::controls::music_library::MusicLibrary;
use crate::controls::playback_control::{PlaybackControl, PlaybackStatus};
use crate::controls::sound_control::SoundControl;
use crate::controls::visualizer::Visualizer;

pub struct AudioSystem {
    library: Arc<Mutex<MusicLibrary>>,
    playback: Arc<Mutex<PlaybackControl>>,
    sound: Arc<Mutex<SoundControl>>,
    sink: Sink,
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    visualizer: Arc<Mutex<Visualizer>>,
}

impl AudioSystem {
    pub fn new(
        library: Arc<Mutex<MusicLibrary>>,
        playback: Arc<Mutex<PlaybackControl>>,
        sound: Arc<Mutex<SoundControl>>,
        visualizer: Arc<Mutex<Visualizer>>,
    ) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        Self {
            library,
            playback,
            sound,
            sink,
            stream,
            stream_handle,
            visualizer,
        }
    }

    /// Play the track at the specified index
    pub fn play_track(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        // Get track path from library
        let track_path = {
            let library = self.library.lock().unwrap();
            if index < library.tracks.len() {
                library.tracks[index].path.clone()
            } else {
                return Err("Invalid track index".into());
            }
        };

        // Decode and play the track
        let source = rodio::Decoder::new(std::fs::File::open(track_path)?)?;
        self.sink.clear();
        self.sink.append(source);
        self.sink.play();

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

        // Apply current sound settings
        self.apply_sound_settings();

        Ok(())
    }

    /// Update playback progress and handle track completion
    pub fn update_playback(&mut self) {
        let mut should_stop = false;

        // Check playback status and update elapsed time
        {
            let mut playback = self.playback.lock().unwrap();
            if playback.status == PlaybackStatus::Playing {
                if let Some(current_track_index) = playback.current_track {
                    let track_duration = self.library.lock().unwrap().tracks[current_track_index]
                        .duration
                        .unwrap_or_default();

                    // Increment elapsed time
                    let current_elapsed = playback.elapsed + Duration::from_millis(16);
                    playback.update_elapsed(current_elapsed);

                    // Check if track has finished
                    if current_elapsed >= track_duration {
                        should_stop = true;
                    }
                }
            }
        }

        // Stop if track has finished
        if should_stop {
            self.stop();
        }

        // Update visualizer (placeholder)
        // You might want to implement actual spectrum analysis here
        if let Ok(mut visualizer) = self.visualizer.lock() {
            // Generate some dummy spectrum data
            let dummy_spectrum = vec![0.0; 32];
            visualizer.update_spectrum(dummy_spectrum);
        }
    }

    /// Apply current sound settings to the audio output
    fn apply_sound_settings(&mut self) {
        let sound = self.sound.lock().unwrap();

        // Set volume
        let volume = sound.volume / 100.0;
        self.sink.set_volume(volume);

        // TODO: Implement more advanced audio processing
        // - Bass and treble adjustments would require more complex audio processing
        // - Balance control might need a custom audio source or additional libraries
    }

    /// Pause current playback
    pub fn pause(&mut self) {
        let mut playback = self.playback.lock().unwrap();
        playback.status = PlaybackStatus::Paused;
        drop(playback);

        self.sink.pause();
    }

    /// Resume paused playback
    pub fn resume(&mut self) {
        let mut playback = self.playback.lock().unwrap();
        playback.status = PlaybackStatus::Playing;
        drop(playback);

        self.sink.play();
    }

    /// Stop current playback
    pub fn stop(&mut self) {
        let mut playback = self.playback.lock().unwrap();
        playback.status = PlaybackStatus::Stopped;
        playback.elapsed = Duration::ZERO;
        playback.current_track = None;
        drop(playback);

        self.sink.stop();
    }

    /// Getter for spectrum data (for visualization)
    pub fn get_spectrum_data(&self) -> Vec<f32> {
        // Placeholder - in a real implementation, this would come from actual audio analysis
        vec![0.0; 32]
    }
}
