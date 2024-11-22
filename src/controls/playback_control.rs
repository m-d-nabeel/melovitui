use std::time::Duration;

#[derive(Default, Clone)]
pub struct PlaybackControl {
    pub status: PlaybackStatus,
    pub current_track: Option<usize>,
    pub elapsed: Duration,
    pub duration: Duration,
}

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
