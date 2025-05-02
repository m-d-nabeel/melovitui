use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use crate::log_error;

use super::sound_control::SoundControl;

#[allow(unused)]
pub struct PlaybackEngine {
    sink: Sink,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    audio_processor: AudioProcessor,
}

impl PlaybackEngine {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        let audio_processor = AudioProcessor::new();

        Ok(Self {
            sink,
            _stream: stream,
            _stream_handle: stream_handle,
            audio_processor,
        })
    }

    pub fn play(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        self.sink.clear();
        let file = File::open(path)?;
        let source = rodio::Decoder::new(file)?;
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
        self.sink.try_seek(new_position)?;
        Ok(())
    }

    pub fn apply_effects(&mut self, sound_control: &SoundControl) {
        // Set volume (0.0 to 1.0 scale for rodio)
        let volume = sound_control.volume() / 100.0;
        self.sink.set_volume(volume);

        // For bass, treble and balance, we would ideally use the AudioProcessor,
        // but since rodio doesn't provide built-in equalization, we'll rely on
        // just setting the volume for now.
        // The AudioProcessor would need to be more complex to implement true
        // equalization and balance control.
    }
}

// Audio processor for more complex sound manipulation
pub struct AudioProcessor {}

#[allow(unused)]
impl AudioProcessor {
    pub fn new() -> Self {
        Self {}
    }

    // This would be the place to implement more complex audio processing
    // like equalization for bass/treble and channel balance
    pub fn process<S: Source>(&self, source: &mut S, settings: &SoundControl)
    where
        <S as std::iter::Iterator>::Item: rodio::Sample,
    {
        // Future implementation would include:
        // 1. Bass boost/cut using low-pass filter
        // 2. Treble boost/cut using high-pass filter
        // 3. Balance adjustment by scaling left/right channels
    }
}
