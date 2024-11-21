use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::state::{AppState, AudioControl, PlaybackStatus};

use super::visualizer::Visualizer;

pub struct AudioSystem {
    app_state: Arc<Mutex<AppState>>,
    sink: Sink,
    #[allow(dead_code)]
    stream: OutputStream,
    #[allow(dead_code)]
    stream_handle: OutputStreamHandle,
    visualizer: Visualizer,
}

impl AudioSystem {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let visualizer = Visualizer::new();

        AudioSystem {
            app_state,
            sink,
            stream,
            stream_handle,
            visualizer,
        }
    }

    /// Play the currently selected track
    pub fn play_track(&mut self) -> Result<(), Box<dyn Error>> {
        let (track_path, track_duration) = {
            let app_state = self.app_state.lock().unwrap();
            if let Some(index) = app_state.playback.current_track {
                let track = &app_state.library.tracks[index];
                (track.path.clone(), track.duration)
            } else {
                return Err("No track selected".into());
            }
        };

        // Perform audio operations outside the lock
        let source = rodio::Decoder::new(std::fs::File::open(track_path)?)?;
        self.sink.clear();
        self.sink.append(source);
        self.sink.play();

        // Update the app state
        let mut app_state = self.app_state.lock().unwrap();
        if let Some(index) = app_state.playback.current_track {
            app_state.playback.status = PlaybackStatus::Playing;
            app_state.playback.elapsed = Duration::ZERO;
            app_state.playback.duration = track_duration.unwrap_or_default();
        }

        Ok(())
    }

    /// Update playback progress and handle track completion
    pub fn update_playback(&mut self) {
        let mut should_stop = false;
        let mut app_state = self.app_state.lock().unwrap();

        if app_state.playback.status == PlaybackStatus::Playing {
            if let Some(current_track_index) = app_state.playback.current_track {
                let track_duration = app_state.library.tracks[current_track_index]
                    .duration
                    .unwrap_or_default();

                // Increment elapsed time
                app_state.playback.elapsed += Duration::from_millis(16); // Assuming 60 FPS update

                // Check if track has finished
                if app_state.playback.elapsed >= track_duration {
                    should_stop = true;
                }
            }
        }

        // Drop the lock before potentially calling stop
        drop(app_state);

        if should_stop {
            self.stop();
        }
    }

    /// Set audio control (volume, bass, treble, balance)
    pub fn set_audio_control(
        &mut self,
        control: AudioControl,
        value: f32,
    ) -> Result<(), Box<dyn Error>> {
        // Update app state
        {
            let mut app_state = self.app_state.lock().unwrap();
            app_state.set_audio_control(control, value);
        }

        // Apply audio control changes
        match control {
            AudioControl::Volume => {
                let volume = {
                    let app_state = self.app_state.lock().unwrap();
                    app_state.audio.volume / 100.0
                };
                self.sink.set_volume(volume);
            }
            AudioControl::Balance => {
                // TODO: Implement balance control if supported by rodio
                // This might require a custom audio source or additional libraries
                eprintln!("Balance control not yet implemented");
            }
            AudioControl::Bass | AudioControl::Treble => {
                // These typically require equalizer support, which is not
                // directly available in rodio
                eprintln!("Equalizer controls not yet implemented");
            }
        }

        Ok(())
    }

    /// Pause current playback
    pub fn pause(&mut self) {
        let mut app_state = self.app_state.lock().unwrap();
        app_state.playback.status = PlaybackStatus::Paused;
        drop(app_state);

        self.sink.pause();
    }

    /// Resume paused playback
    pub fn resume(&mut self) {
        let mut app_state = self.app_state.lock().unwrap();
        app_state.playback.status = PlaybackStatus::Playing;
        drop(app_state);

        self.sink.play();
    }

    /// Stop current playback
    pub fn stop(&mut self) {
        let mut app_state = self.app_state.lock().unwrap();
        app_state.playback.status = PlaybackStatus::Stopped;
        app_state.playback.elapsed = Duration::ZERO;
        drop(app_state);

        self.sink.stop();
    }
}
