use crate::log_debug;

/// Audio settings with well-defined constraints for controlling sound characteristics.
///
/// This struct manages all user-adjustable audio parameters:
/// - Volume: Controls overall playback loudness (0-100)
/// - Bass: Enhances or reduces low frequencies (-100 to 100)
/// - Treble: Enhances or reduces high frequencies (-100 to 100)
/// - Balance: Controls left/right channel balance (-100 to 100)
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
