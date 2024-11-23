use crossbeam::channel;
use parking_lot::Mutex;
use rodio::Source;
use rustfft::{num_complex::Complex, FftPlanner};
use std::{f32::consts::PI, sync::Arc, time::Duration};

use crate::audio_system::SoundControl;

const FFT_SIZE: usize = 2048;
const VISUALIZER_BANDS: usize = 32;
// Standard corner frequencies for bass and treble shelving filters
const BASS_CORNER_FREQ: f32 = 250.0; // Common bass corner frequency
const TREBLE_CORNER_FREQ: f32 = 4000.0; // Common treble corner frequency

/// A source that processes audio with EQ and effects
pub struct ProcessedSource<I> {
    inner: I,
    state: Arc<Mutex<SoundControl>>,
    spectrum_tx: channel::Sender<Vec<f32>>,
    fft_planner: FftPlanner<f32>,
    buffer: Vec<f32>,
    prev_samples: Vec<f32>, // For smoothing transitions
}

pub struct AudioEngine {
    state: Arc<Mutex<SoundControl>>,
    spectrum_rx: channel::Receiver<Vec<f32>>,
    spectrum_tx: channel::Sender<Vec<f32>>,
}

impl AudioEngine {
    pub fn new(state: Arc<Mutex<SoundControl>>) -> Self {
        let (spectrum_tx, spectrum_rx) = channel::bounded(2);

        Self {
            state,
            spectrum_rx,
            spectrum_tx,
        }
    }

    pub fn process_source<I>(&self, inner: I) -> ProcessedSource<I>
    where
        I: Source<Item = f32>,
    {
        ProcessedSource {
            inner,
            state: self.state.clone(),
            spectrum_tx: self.spectrum_tx.clone(),
            fft_planner: FftPlanner::new(),
            buffer: Vec::with_capacity(FFT_SIZE),
            prev_samples: vec![0.0; FFT_SIZE],
        }
    }

    pub fn get_spectrum_frame(&self) -> Option<Vec<f32>> {
        self.spectrum_rx.try_recv().ok()
    }
}

impl<I> ProcessedSource<I>
where
    I: Source<Item = f32>,
{
    fn apply_shelving_filter(&self, freq: f32, corner_freq: f32, gain_db: f32) -> f32 {
        // Convert gain from -100..100 range to dB (-12..12 dB range)
        let normalized_gain = (gain_db / 100.0) * 12.0;

        let s = 2.0; // Shelf slope (steepness)
        let _w0 = 2.0 * PI * corner_freq;
        let a = 10.0_f32.powf(normalized_gain / 40.0); // Convert dB to linear gain

        let ratio = freq / corner_freq;
        let phase = (ratio * PI / 2.0).min(PI / 2.0);

        // Calculate shelf response
        let transition = (phase / (PI / 2.0)).powf(2.0 * s);

        1.0 + (a - 1.0) * transition
    }

    fn process_buffer(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = input.to_vec();
        let state = self.state.lock();

        // Convert to complex for FFT
        let mut complex: Vec<Complex<f32>> = input.iter().map(|&x| Complex::new(x, 0.0)).collect();

        // Apply FFT
        let fft = self.fft_planner.plan_fft_forward(complex.len());
        fft.process(&mut complex);

        // Apply EQ using shelving filters
        let sample_rate = self.inner.sample_rate();
        let freq_step = sample_rate as f32 / complex.len() as f32;

        for (i, sample) in complex.iter_mut().enumerate() {
            let freq = i as f32 * freq_step;

            // Apply bass shelf
            let bass_gain = self.apply_shelving_filter(freq, BASS_CORNER_FREQ, state.bass());

            // Apply treble shelf
            let treble_gain = self.apply_shelving_filter(freq, TREBLE_CORNER_FREQ, state.treble());

            // Combine both filters
            let total_gain = bass_gain * treble_gain;
            *sample *= Complex::new(total_gain, 0.0);
        }

        // Generate spectrum data for visualizer with improved scaling
        let spectrum = self.generate_spectrum_data(&complex);
        let _ = self.spectrum_tx.try_send(spectrum);

        // Inverse FFT
        let ifft = self.fft_planner.plan_fft_inverse(complex.len());
        ifft.process(&mut complex);

        // Apply volume, balance, and smoothing
        let smoothing_factor = 0.15; // Smoothing coefficient
        for (i, sample) in output.iter_mut().enumerate() {
            let processed = complex[i].re / complex.len() as f32;

            // Apply balance
            let balance_factor = if i % 2 == 0 {
                1.0 - state.balance().max(0.0) / 100.0
            } else {
                1.0 + state.balance().min(0.0) / 100.0
            };

            // Calculate target sample value
            let target = processed * (state.volume() / 100.0) * balance_factor;

            // Apply smoothing between previous and current sample
            *sample = self.prev_samples[i] + (target - self.prev_samples[i]) * smoothing_factor;
            self.prev_samples[i] = *sample;
        }

        output
    }

    fn generate_spectrum_data(&self, complex_data: &[Complex<f32>]) -> Vec<f32> {
        let mut spectrum = vec![0.0; VISUALIZER_BANDS];
        let band_size = complex_data.len() / VISUALIZER_BANDS;

        for (i, band) in spectrum.iter_mut().enumerate() {
            let start = i * band_size;
            let end = start + band_size;

            // Calculate magnitude with improved scaling
            let raw_magnitude = complex_data[start..end]
                .iter()
                .map(|c| (c.re * c.re + c.im * c.im).sqrt())
                .sum::<f32>()
                / band_size as f32;

            // Apply logarithmic scaling and normalization
            *band = (raw_magnitude + 1.0).log10() * 20.0;
        }

        spectrum
    }
}

// Source trait implementation remains the same
impl<I> Source for ProcessedSource<I>
where
    I: Source<Item = f32>,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

impl<I> Iterator for ProcessedSource<I>
where
    I: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next sample first
        let next_sample = self.inner.next()?;

        // Now push it to buffer
        self.buffer.push(next_sample);

        if self.buffer.len() >= FFT_SIZE {
            // Create a copy of the buffer for processing
            let buffer_copy = self.buffer.clone();
            // Process the copied buffer
            let processed = self.process_buffer(&buffer_copy);
            // Clear the original buffer
            self.buffer.clear();
            // Return the first processed sample
            Some(processed[0])
        } else {
            // Return the unprocessed sample
            Some(next_sample)
        }
    }
}

//// Integration with your AudioSystem
//impl AudioSystem {
//    pub fn with_engine(
//        library: Arc<Mutex<MusicLibrary>>,
//        playback: Arc<Mutex<PlaybackControl>>,
//        visualizer: Arc<Mutex<Visualizer>>,
//    ) -> Result<(Self, AudioEngine), Box<dyn Error>> {
//        let system = Self::new(library, playback.clone(), visualizer)?;
//        let engine = AudioEngine::new();
//        Ok((system, engine))
//    }
//
//    // Modified play_track method
//    pub fn play_track_with_processing(
//        &mut self,
//        track_index: Option<usize>,
//        engine: &AudioEngine,
//    ) -> Result<(), Box<dyn Error>> {
//        let index = track_index.unwrap_or_else(|| self.library.lock().selected_index.unwrap_or(0));
//
//        let track_path = {
//            let library = self.library.lock();
//            library
//                .tracks
//                .get(index)
//                .ok_or_else(|| format!("Invalid track index: {}", index))?
//                .path
//                .clone()
//        };
//
//        // Decode and process the track
//        let file = std::fs::File::open(&track_path)?;
//        let source = rodio::Decoder::new(file)?;
//        let processed_source = engine.process_source(source);
//
//        self.sink.clear();
//        self.sink.append(processed_source);
//        self.sink.play();
//
//        // Update playback state
//        {
//            let mut playback = self.playback.lock();
//            if let Some(duration) = self
//                .library
//                .lock()
//                .tracks
//                .get(index)
//                .and_then(|track| track.duration)
//            {
//                playback.start(index, duration);
//            }
//        }
//
//        Ok(())
//    }
//}
