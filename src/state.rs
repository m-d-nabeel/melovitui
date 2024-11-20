use std::path::PathBuf;
use std::time::Duration;

/// Main application state container
#[derive(Default, Clone)]
pub struct AppState {
    pub library: LibraryState,
    pub audio: AudioState,
    pub playback: PlaybackState,
}

/// Library state for music list
#[derive(Default, Clone)]
pub struct LibraryState {
    pub tracks: Vec<Track>,
    pub selected_index: Option<usize>,
    pub current_dir: PathBuf,
}

/// Audio controls and visualization state
#[derive(Default, Clone)]
pub struct AudioState {
    pub volume: f32,             // 0.0 to 100.0
    pub bass: f32,               // 0.0 to 100.0
    pub treble: f32,             // 0.0 to 100.0
    pub balance: f32,            // -100.0 (left) to 100.0 (right)
    pub spectrum_data: Vec<f32>, // Visualization data
}

/// Playback state for current track
#[derive(Default, Clone)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub current_track: Option<usize>, // Index into library tracks
    pub elapsed: Duration,
    pub duration: Duration,
}

/// Track information
#[derive(Clone, Debug)]
pub struct Track {
    pub title: String,
    pub artist: Option<String>,
    pub path: PathBuf,
    pub duration: Option<Duration>,
}

/// Playback status enum
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl Default for PlaybackStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

// Implementation block for AppState
impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update spectrum data for visualization
    pub fn update_spectrum(&mut self, data: Vec<f32>) {
        self.audio.spectrum_data = data;
    }

    /// Set the current track and initialize playback
    pub fn set_current_track(&mut self, index: usize) -> Option<&Track> {
        if index < self.library.tracks.len() {
            self.playback.current_track = Some(index);
            self.playback.status = PlaybackStatus::Playing;
            self.playback.elapsed = Duration::default();
            self.playback.duration = self.library.tracks[index].duration.unwrap_or_default();
            Some(&self.library.tracks[index])
        } else {
            None
        }
    }

    /// Update audio control values
    pub fn set_audio_control(&mut self, control: AudioControl, value: f32) {
        match control {
            AudioControl::Volume => self.audio.volume = value.clamp(0.0, 100.0),
            AudioControl::Bass => self.audio.bass = value.clamp(0.0, 100.0),
            AudioControl::Treble => self.audio.treble = value.clamp(0.0, 100.0),
            AudioControl::Balance => self.audio.balance = value.clamp(-100.0, 100.0),
        }
    }
}

/// Audio control types for the UI
#[derive(Debug, Clone, Copy)]
pub enum AudioControl {
    Volume,
    Bass,
    Treble,
    Balance,
}

// Implementation for Track
impl Track {
    pub fn new(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self {
            title: filename,
            artist: None,
            path,
            duration: None,
        }
    }

    pub fn with_metadata(
        path: PathBuf,
        title: String,
        artist: Option<String>,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            title,
            artist,
            path,
            duration,
        }
    }
}
