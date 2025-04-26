/// Stores frequency spectrum data for audio visualization.
///
/// This struct contains the results of Fast Fourier Transform (FFT) analysis
/// of audio data, providing the frequency-domain representation used for
/// creating audio visualizations.
#[derive(Default)]
pub struct Spectrum {
    pub inner: Vec<f32>,
    pub size: usize,
    pub fps: usize,
}
