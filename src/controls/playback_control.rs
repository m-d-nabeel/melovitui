use std::time::Duration;

/// Manages playback state information including track selection, position, and status.
///
/// This struct tracks the current playback state of the audio system, including:
/// - Current playing status (playing, paused, stopped)
/// - Selected track index
/// - Elapsed time within the current track
/// - Total duration of the current track
#[derive(Default, Clone)]
pub struct PlaybackControl {
    pub status: PlaybackStatus,
    pub current_track: Option<usize>,
    pub elapsed: Duration,
    pub duration: Duration,
}

/// Represents the current playback status of the audio system.
///
/// This enum defines the three possible states of playback:
/// - Stopped: No playback is active
/// - Playing: Audio is currently playing
/// - Paused: Playback is temporarily suspended but position is maintained
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PlaybackStatus {
    #[default]
    Stopped,
    Playing,
    Paused,
}

impl PlaybackControl {
    pub fn start(&mut self, track_index: usize, total_duration: Duration) {
        self.current_track = Some(track_index);
        self.status = PlaybackStatus::Playing;
        self.elapsed = Duration::ZERO;
        self.duration = total_duration;
    }

    pub fn update_elapsed(&mut self, time: Duration) {
        self.elapsed = time.min(self.duration);
    }
}
