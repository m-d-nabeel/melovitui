#[derive(Default, Clone)]
pub struct SoundControl {
    pub volume: f32,
    pub bass: f32,
    pub treble: f32,
    pub balance: f32,
}

impl SoundControl {
    pub fn set_volume(&mut self, value: f32) {
        self.volume = value.clamp(0.0, 100.0);
    }

    pub fn set_bass(&mut self, value: f32) {
        self.bass = value.clamp(0.0, 100.0);
    }

    pub fn set_treble(&mut self, value: f32) {
        self.treble = value.clamp(0.0, 100.0);
    }

    pub fn set_balance(&mut self, value: f32) {
        self.balance = value.clamp(-100.0, 100.0);
    }
}

