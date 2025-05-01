use std::{error::Error, fs::File, io::BufReader, path::Path};

use rodio::{Decoder, Source};
use rustfft::{num_complex::Complex, FftDirection, FftPlanner};

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

impl Spectrum {
    pub fn fft(path: impl AsRef<Path>) -> Result<Spectrum, Box<dyn Error>> {
        let src = File::open(path).unwrap();

        let source = Decoder::new_mp3(BufReader::new(src)).unwrap();

        let samples = source.convert_samples::<f32>();

        let ch = samples.channels() as usize;
        let rate = samples.sample_rate();
        assert!(rate % 60 == 0); // assume sample rate is divisible by 60 so that we can stream each frame 60 times per second
        let size = (rate / 60) as usize;
        let msize = size * ch;

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft(size, FftDirection::Forward);

        let mut slices = vec![];

        let hamming = apodize::hamming_iter(size)
            .map(|n| n as f32)
            .collect::<Vec<f32>>();

        let samples = samples.buffered();

        // TODO buffer not needed bc.: process(&mut slices[a..b])
        let mut buffer = vec![];
        for (k, b) in samples.enumerate() {
            if k != 0 && k % msize == 0 {
                fft.process(&mut buffer);
                if buffer.len() != size {
                    break;
                }
                slices.append(&mut buffer);
            }
            if k % ch == 0 {
                buffer.push(Complex {
                    re: b * hamming[(k % msize) / 2] as f32,
                    im: 0.0,
                });
            }
        }

        let out = slices
            .into_iter()
            .map(|v| (v.re * v.re + v.im * v.im).sqrt())
            .collect::<Vec<f32>>();

        // in buffer, frames are every `size`, 60 frames = 1sec

        Ok(Spectrum {
            inner: out,
            size,
            fps: 60,
        })
    }
}
