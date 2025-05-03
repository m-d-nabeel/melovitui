use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use crate::log_error;

use super::sound_control::SoundControl;

pub struct AudioEngine {
    sink: Sink,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        Ok(Self {
            sink,
            _stream: stream,
            _stream_handle: stream_handle,
        })
    }

    pub fn play(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        self.sink.clear();
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let source = rodio::Decoder::new(reader)?;
        self.sink.append(source);
        self.sink.play();
        Ok(())
    }

    pub fn pause(&mut self) {
        self.sink.pause();
    }

    pub fn resume(&mut self) {
        self.sink.play();
    }

    // TODO: Handler error better
    pub fn stop(&mut self) {
        match self.sink.try_seek(Duration::ZERO) {
            Ok(_) => {
                self.sink.pause();
            }
            Err(e) => log_error!("{}", e),
        }
    }

    pub fn seek_control(&mut self, new_position: Duration) -> Result<(), Box<dyn Error>> {
        log::debug!("Attempting to seek to position: {:?}", new_position);

        match self.sink.try_seek(new_position) {
            Ok(_) => {
                log::debug!("Successfully sought to position: {:?}", new_position);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to seek to position {:?}: {:?}", new_position, e);
                Err(Box::new(e))
            }
        }
    }

    pub fn apply_effects(&mut self, sound_control: &SoundControl) {
        // Set volume (0.0 to 1.0 scale for rodio)
        let volume = sound_control.volume() / 100.0;
        self.sink.set_volume(volume);

        // // Set pitch (map -100..100 to -1.0..1.0 range)
        // let pitch = 1.0 + (sound_control.pitch() / 100.0);
        // self.sink.set_speed(pitch);

        let speed = Self::calc_playback_speed(sound_control.pitch());
        self.sink.set_speed(speed);

        // For bass, treble and pitch, we would ideally use the AudioProcessor,
        // but since rodio doesn't provide built-in equalization, we'll rely on
        // just setting the volume for now.
        // The AudioProcessor would need to be more complex to implement true
        // equalization and pitch control.
    }

    pub fn get_current_pos(&self) -> std::time::Duration {
        self.sink.get_pos()
    }

    pub fn is_sink_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn calc_playback_speed(pitch: f32) -> f32 {
        let pitch_factor = pitch / 100.0;
        let semitone_range = 24.0;
        let speed = 2.0f32.powf(pitch_factor * semitone_range / 12.0);
        speed.clamp(0.5, 2.0)
    }
}
