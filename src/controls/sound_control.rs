use crate::log_debug;

/// Audio settings with well-defined constraints for controlling sound characteristics.
///
/// This struct manages all user-adjustable audio parameters:
/// - Volume: Controls overall playback loudness (0-100)
/// - Bass: Enhances or reduces low frequencies (0-100)
/// - Treble: Enhances or reduces high frequencies (0-100)
/// - Pitch: Controls audio pitch adjustment (-100-100)
#[derive(Debug, Clone)]
pub struct SoundControl {
    volume: f32,
    bass: f32,
    treble: f32,
    pitch: f32,
}

impl Default for SoundControl {
    fn default() -> Self {
        Self {
            volume: 50.0,
            bass: 0.0,
            treble: 0.0,
            pitch: 0.0,
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
        log_debug!("Volume adjusted to {}", self.volume);
    }

    /// Adjusts the bass by a delta and clamps it within the valid range
    pub fn adjust_bass(&mut self, delta: f32) {
        self.bass = (self.bass + delta).clamp(0.0, 100.0);
        log_debug!("Bass adjusted to {}", self.bass);
    }

    /// Adjusts the treble by a delta and clamps it within the valid range
    pub fn adjust_treble(&mut self, delta: f32) {
        self.treble = (self.treble + delta).clamp(0.0, 100.0);
        log_debug!("Treble adjusted to {}", self.treble);
    }

    /// Adjusts the pitch by a delta and clamps it within the valid range
    pub fn adjust_pitch(&mut self, delta: f32) {
        self.pitch = (self.pitch + delta).clamp(-100.0, 100.0);
        log_debug!("Pitch adjusted to {}", self.pitch);
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

    /// Getter for pitch
    pub fn pitch(&self) -> f32 {
        self.pitch
    }
}
