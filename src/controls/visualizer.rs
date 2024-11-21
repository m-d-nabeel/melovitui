#[derive(Default, Clone, Debug)]
pub struct Visualizer {
    pub spectrum_data: Vec<f32>,
}

impl Visualizer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update_spectrum(&mut self, data: Vec<f32>) {
        self.spectrum_data = data;
    }

    pub fn clear_spectrum(&mut self) {
        self.spectrum_data.clear();
    }
}
