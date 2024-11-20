use crate::state::{AppState, PlaybackStatus};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::time::Duration;

pub struct AudioSystem {
    app_state: AppState,
    sink: Sink,
    stream_handle: OutputStreamHandle,
    spectrum_analyzer: SpectrumAnalyzer,
}

impl AudioSystem {
    pub fn new(app_state: &AppState) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let spectrum_analyzer = SpectrumAnalyzer::new();

        AudioSystem {
            app_state: app_state.clone(),
            sink,
            stream_handle,
            spectrum_analyzer,
        }
    }

    pub fn play_track(&mut self) {
        if let Some(index) = self.app_state.playback.current_track {
            let track = &self.app_state.library.tracks[index];
            let source =
                rodio::Decoder::new(std::fs::File::open(track.path.clone()).unwrap()).unwrap();
            self.sink.append(source);
            self.app_state.playback.status = PlaybackStatus::Playing;
        }
    }

    pub fn pause(&mut self) {
        self.app_state.playback.status = PlaybackStatus::Paused;
        self.sink.pause();
    }

    pub fn resume(&mut self) {
        self.app_state.playback.status = PlaybackStatus::Playing;
        self.sink.play();
    }

    pub fn stop(&mut self) {
        self.app_state.playback.status = PlaybackStatus::Stopped;
        self.sink.stop();
    }

    pub fn seek(&mut self, position: Duration) {
        let _ = self.sink.try_seek(position);
    }

    pub fn get_playback_position(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn get_playback_duration(&self) -> Duration {
        if let Some(index) = self.app_state.playback.current_track {
            self.app_state.library.tracks[index]
                .duration
                .unwrap_or_default()
        } else {
            Duration::default()
        }
    }

    pub fn update_playback(&mut self, delta: std::time::Duration) {
        self.app_state.playback.elapsed += delta;
    }

    pub fn update_audio_controls(&mut self) {
        self.sink.set_volume(self.app_state.audio.volume / 100.0);
        // Update bass, treble, and balance
    }

    pub fn get_spectrum_data(&mut self) -> Vec<f32> {
        self.spectrum_analyzer.get_spectrum_data()
    }
}

struct SpectrumAnalyzer {
    // FFT and other audio processing logic
}

impl SpectrumAnalyzer {
    fn new() -> Self {
        SpectrumAnalyzer {}
    }

    fn get_spectrum_data(&mut self) -> Vec<f32> {
        // Analyze audio stream and return spectrum data
        vec![0.0; 128]
    }
}
