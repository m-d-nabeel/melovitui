use core::f64;
use std::process::Command;

use ratatui::layout::Alignment;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{
        canvas::{Canvas, Context, Line, Rectangle},
        Block, BorderType, Borders,
    },
    Frame,
};
use rustfft::num_complex::ComplexFloat;

pub struct VisualizerUI {
    style: VisualizerStyle,
}

pub struct VisualizerStyle {
    pub block_style: Style,
    pub peak_color: Color,
    pub particle_color: Color,
    pub wave_color: Color,
}

impl Default for VisualizerStyle {
    fn default() -> Self {
        Self {
            block_style: Style::default()
                .bg(Color::Black)
                .fg(Color::Rgb(150, 160, 255)),
            peak_color: Color::Rgb(180, 140, 230),
            particle_color: Color::Rgb(255, 255, 255),
            wave_color: Color::Rgb(138, 43, 226),
        }
    }
}

type CanvasPainter<'a> = Box<dyn Fn(&mut Context) + 'a>;

impl VisualizerUI {
    pub fn new() -> Self {
        Self {
            style: VisualizerStyle::default(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, spectrum: Vec<f32>, canvas_type: usize) {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        // Define the block with improved readability
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .title("Spectrum Visualizer")
            .style(self.style.block_style);

        // Render the outer block and compute the inner area for the canvas
        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Select the appropriate canvas based on `canvas_type`
        let canvas = match canvas_type {
            0 => self.get_canvas_cava(inner_area, &spectrum, time),
            1 => self.get_canvas_1(inner_area, &spectrum, time),
            2 => self.get_canvas_2(inner_area, &spectrum, time),
            3 => self.get_canvas_3(inner_area, &spectrum, time),
            4 => self.get_canvas_4(inner_area, &spectrum, time),
            _ => {
                if is_cava_installed() {
                    self.get_canvas_cava(inner_area, &spectrum, time)
                } else {
                    self.get_canvas_4(inner_area, &spectrum, time)
                }
            }
        };

        // Render the selected canvas
        frame.render_widget(canvas, inner_area);
    }
}

impl VisualizerUI {
    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_1<'a>(
        &'a self,
        inner_area: Rect,
        spectrum: &'a [f32],
        time: f64,
    ) -> Canvas<CanvasPainter<'a>> {
        Canvas::default()
            .marker(symbols::Marker::Braille)
            .x_bounds([0.0, inner_area.width.into()])
            .y_bounds([0.0, inner_area.height.into()])
            .paint(Box::new(move |ctx| {
                let center_y = inner_area.height as f64 / 2.0;
                let max_height = inner_area.height as f64 / 3.0;
                let width = inner_area.width as f64;
                let height = inner_area.height as f64;

                // Fill background with a subtle gradient
                ctx.draw(&Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                    color: Color::Rgb(10, 10, 20),
                });

                // Use static data for smoothing
                use std::sync::Mutex;
                use std::sync::OnceLock;

                static SMOOTHED_VALUES: OnceLock<Mutex<Vec<f64>>> = OnceLock::new();
                static PEAKS: OnceLock<Mutex<Vec<f64>>> = OnceLock::new();

                // Number of bands to visualize (half the width to allow for mirroring)
                let num_bands = (inner_area.width as usize / 2).min(spectrum.len());

                let smoothed = SMOOTHED_VALUES.get_or_init(|| Mutex::new(vec![0.0; num_bands]));
                let peaks = PEAKS.get_or_init(|| Mutex::new(vec![0.0; num_bands]));

                // Update smoothed values and peaks
                {
                    let mut smoothed_data = smoothed.lock().unwrap();
                    let mut peaks_data = peaks.lock().unwrap();

                    for (i, &value) in spectrum.iter().enumerate().take(num_bands) {
                        // Apply smoothing factor (lower = smoother)
                        let alpha = 0.15;
                        let normalized_value = f64::from(value).powf(0.7);
                        smoothed_data[i] =
                            smoothed_data[i] * (1.0 - alpha) + normalized_value * alpha;

                        // Update peak values with slow decay
                        if smoothed_data[i] > peaks_data[i] {
                            peaks_data[i] = smoothed_data[i];
                        } else {
                            peaks_data[i] *= 0.98; // Slow decay for peaks
                        }
                    }
                }

                let smoothed_values = smoothed.lock().unwrap().clone();
                let peak_values = peaks.lock().unwrap().clone();

                // Draw smooth centerline with wave effect
                let wave_speed = 0.3; // Slower wave speed
                for x in 0..inner_area.width {
                    let wave = (x as f64 * 0.05 + time * wave_speed).sin() * 2.0;
                    ctx.draw(&Line {
                        x1: x as f64,
                        y1: center_y + wave - 0.5,
                        x2: x as f64,
                        y2: center_y + wave + 0.5,
                        color: Color::Rgb(40, 40, 60),
                    });
                }

                // Mirror effect visualization
                for i in 0..num_bands {
                    let x = i as f64;
                    let mirror_x = inner_area.width as f64 - i as f64 - 1.0;

                    // Use smoothed values for visualization
                    let height = smoothed_values[i] * max_height;
                    let peak_height = peak_values[i] * max_height;

                    // Skip very small values
                    if height < 0.5 {
                        continue;
                    }

                    // Calculate wave offsets with slower animation
                    let wave_freq = 0.8; // Slower frequency
                    let upper_wave = (time * wave_freq + i as f64 * 0.1).sin() * 1.0;
                    let lower_wave =
                        (time * wave_freq + i as f64 * 0.1 + std::f64::consts::PI).sin() * 1.0;

                    let upper_y = center_y - height + upper_wave;
                    let lower_y = center_y + height + lower_wave;

                    // Draw main visualization lines
                    for offset in [-1.0, 0.0, 1.0].iter() {
                        let alpha = 1.0 - offset.abs() * 0.3;
                        let color = match self.style.wave_color {
                            Color::Rgb(r, g, b) => Color::Rgb(
                                (r as f64 * alpha) as u8,
                                (g as f64 * alpha) as u8,
                                (b as f64 * alpha) as u8,
                            ),
                            _ => self.style.wave_color,
                        };

                        // Upper lines
                        ctx.draw(&Line {
                            x1: x + offset,
                            y1: center_y,
                            x2: x + offset,
                            y2: upper_y,
                            color,
                        });
                        ctx.draw(&Line {
                            x1: mirror_x + offset,
                            y1: center_y,
                            x2: mirror_x + offset,
                            y2: upper_y,
                            color,
                        });

                        // Lower lines
                        ctx.draw(&Line {
                            x1: x + offset,
                            y1: center_y,
                            x2: x + offset,
                            y2: lower_y,
                            color,
                        });
                        ctx.draw(&Line {
                            x1: mirror_x + offset,
                            y1: center_y,
                            x2: mirror_x + offset,
                            y2: lower_y,
                            color,
                        });
                    }

                    // Draw peak indicators
                    if peak_height > height * 1.05 {
                        // Use a lighter version of the wave color for peaks
                        let peak_color = match self.style.wave_color {
                            Color::Rgb(r, g, b) => Color::Rgb(
                                r.saturating_add(40),
                                g.saturating_add(40),
                                b.saturating_add(40),
                            ),
                            _ => self.style.peak_color,
                        };

                        let peak_upper_y = center_y - peak_height;
                        let peak_lower_y = center_y + peak_height;

                        // Upper peak
                        ctx.draw(&Rectangle {
                            x,
                            y: peak_upper_y,
                            width: 1.0,
                            height: 1.0,
                            color: peak_color,
                        });
                        ctx.draw(&Rectangle {
                            x: mirror_x,
                            y: peak_upper_y,
                            width: 1.0,
                            height: 1.0,
                            color: peak_color,
                        });

                        // Lower peak
                        ctx.draw(&Rectangle {
                            x,
                            y: peak_lower_y,
                            width: 1.0,
                            height: 1.0,
                            color: peak_color,
                        });
                        ctx.draw(&Rectangle {
                            x: mirror_x,
                            y: peak_lower_y,
                            width: 1.0,
                            height: 1.0,
                            color: peak_color,
                        });
                    }

                    // Add particles at high intensity points - use slower animation
                    if smoothed_values[i] > 0.5 {
                        let particle_color = self.style.particle_color;
                        let particle_freq = 2.0; // Slower frequency
                        let particle_y = upper_y + (time * particle_freq + i as f64).sin() * 3.0;

                        ctx.draw(&Rectangle {
                            x,
                            y: particle_y,
                            width: 1.0,
                            height: 1.0,
                            color: particle_color,
                        });

                        ctx.draw(&Rectangle {
                            x: mirror_x,
                            y: particle_y,
                            width: 1.0,
                            height: 1.0,
                            color: particle_color,
                        });
                    }
                }
            }))
    }

    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_cava<'a>(
        &'a self,
        inner_area: Rect,
        _spectrum: &'a [f32], // Not used with CAVA
        time: f64,
    ) -> Canvas<CanvasPainter<'a>> {
        Canvas::default()
            .marker(symbols::Marker::Braille)
            .x_bounds([0.0, inner_area.width.into()])
            .y_bounds([0.0, inner_area.height.into()])
            .paint(Box::new(move |ctx| {
                use std::io::{BufReader, Read};
                use std::process::{Command, Stdio};
                use std::sync::Mutex;
                use std::sync::OnceLock;
                use std::thread;

                if !is_cava_installed() {
                    return;
                }

                static CAVA_DATA: OnceLock<Mutex<Vec<f32>>> = OnceLock::new();
                static CAVA_RUNNING: OnceLock<Mutex<bool>> = OnceLock::new();

                let cava_data =
                    CAVA_DATA.get_or_init(|| Mutex::new(vec![0.0; inner_area.width as usize]));
                let cava_running = CAVA_RUNNING.get_or_init(|| Mutex::new(false));

                if !*cava_running.lock().unwrap() {
                    let mut guard = cava_running.lock().unwrap();
                    if !*guard {
                        thread::spawn(move || {
                            let config_content = format!(
                                r#"
                            [general]
                            bars = {}
                            framerate = 60
                            autosens = 1

                            [input]
                            method = pulse
                            source = auto

                            [output]
                            method = raw
                            raw_target = /dev/stdout
                            data_format = binary
                            bit_format = 8bit
                            orientation = top

                            [color]
                            gradient = 1
                            gradient_count = 2
                            gradient_color_1 = '#0099ff'
                            gradient_color_2 = '#ff3399'

                            [smoothing]
                            monstercat = 1
                            noise_reduction = 0.77

                            [eq]
                            1 = 1
                            2 = 1
                            3 = 1
                            4 = 1
                            5 = 1
                            "#,
                                inner_area.width
                            );

                            let temp_dir = std::env::temp_dir();
                            let config_path = temp_dir.join("melovitui_cava.conf");
                            std::fs::write(&config_path, config_content).unwrap();

                            let mut cava = Command::new("cava")
                                .arg("-p")
                                .arg(&config_path)
                                .stdout(Stdio::piped())
                                .spawn()
                                .unwrap();

                            *cava_running.lock().unwrap() = true;

                            let stdout = cava.stdout.take().unwrap();
                            let mut reader = BufReader::new(stdout);
                            let num_bars = inner_area.width as usize;
                            let mut buffer = vec![0u8; num_bars];

                            while reader.read_exact(&mut buffer).is_ok() {
                                let mut data = cava_data.lock().unwrap();
                                for (i, &val) in buffer.iter().enumerate() {
                                    if i < num_bars {
                                        data[i] = val as f32 / 255.0;
                                    }
                                }

                                thread::sleep(std::time::Duration::from_millis(16));
                            }

                            *cava_running.lock().unwrap() = false;
                            let _ = cava.wait();
                        });

                        *guard = true;
                    }
                }

                // Rendering
                let width = inner_area.width as f64;
                let height = inner_area.height as f64;

                ctx.draw(&Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                    color: Color::Black,
                });

                let bar_values = {
                    let data_guard = cava_data.lock().unwrap();
                    data_guard.clone()
                };

                let num_bars = bar_values.len();
                let bar_width = width / num_bars as f64;

                for (i, &value) in bar_values.iter().enumerate() {
                    let x = i as f64 * bar_width;
                    let bar_height = value as f64 * height;

                    // Color oscillation
                    let color_phase = i as f64 / num_bars as f64 + time * 0.1;
                    let r = ((color_phase * 2.0).sin() * 0.5 + 0.5) * 255.0;
                    let g = ((color_phase * 2.0 + 2.0).sin() * 0.5 + 0.5) * 255.0;
                    let b = ((color_phase * 2.0 + 4.0).sin() * 0.5 + 0.5) * 255.0;
                    let color = Color::Rgb(r as u8, g as u8, b as u8);

                    // Main bar (BOTTOM-UP)
                    ctx.draw(&Rectangle {
                        x,
                        y: 0.0,
                        width: bar_width - 1.0,
                        height: bar_height,
                        color,
                    });

                    // Peak line
                    let peak_y = bar_height;
                    if peak_y < height {
                        // Use a lighter version of the bar color for peaks
                        let peak_color = Color::Rgb(
                            (r as u16 + 60).min(255) as u8,
                            (g as u16 + 60).min(255) as u8,
                            (b as u16 + 60).min(255) as u8,
                        );

                        ctx.draw(&Rectangle {
                            x,
                            y: peak_y,
                            width: bar_width - 1.0,
                            height: 1.0,
                            color: peak_color,
                        });
                    }
                }
            }))
    }

    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_2<'a>(
        &'a self,
        inner_area: Rect,
        spectrum: &'a [f32],
        time: f64,
    ) -> Canvas<CanvasPainter<'a>> {
        Canvas::default()
            .marker(symbols::Marker::Braille)
            .x_bounds([0.0, inner_area.width.into()])
            .y_bounds([0.0, inner_area.height.into()])
            .paint(Box::new(move |ctx| {
                let width = inner_area.width as f64;
                let height = inner_area.height as f64;
                let center_x = width / 2.0;

                // Fill background
                ctx.draw(&Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                    color: Color::Rgb(10, 10, 20),
                });

                // Bar resolution - use half the width since we'll mirror
                // Increased density by using more of the available spectrum data
                let num_samples = spectrum.len().min(inner_area.width as usize / 3);
                // Adjusted ratio for narrower bars and less spacing to increase density
                let bar_width = (width / 2.0) / num_samples as f64 * 0.7;
                let bar_spacing = (width / 2.0) / num_samples as f64 * 0.3;

                use std::sync::Mutex;
                use std::sync::OnceLock;

                static PEAKS: OnceLock<Mutex<Vec<f64>>> = OnceLock::new();
                static SMOOTHED_VALUES: OnceLock<Mutex<Vec<f64>>> = OnceLock::new();

                let peaks = PEAKS.get_or_init(|| Mutex::new(vec![0.0; num_samples]));
                let smoothed = SMOOTHED_VALUES.get_or_init(|| Mutex::new(vec![0.0; num_samples]));

                {
                    let mut smoothed_data = smoothed.lock().unwrap();
                    let mut peaks_data = peaks.lock().unwrap();

                    for (i, &value) in spectrum.iter().enumerate().take(num_samples) {
                        let alpha = 0.15;
                        let normalized_value = f64::from(value).powf(0.6);
                        smoothed_data[i] =
                            smoothed_data[i] * (1.0 - alpha) + normalized_value * alpha;

                        if smoothed_data[i] > peaks_data[i] {
                            peaks_data[i] = smoothed_data[i];
                        } else {
                            peaks_data[i] *= 0.97;
                        }
                    }
                }

                let smoothed_values = smoothed.lock().unwrap().clone();
                let peak_values = peaks.lock().unwrap().clone();

                for i in 0..num_samples {
                    let bar_height = smoothed_values[i] * height;
                    let peak_height = peak_values[i] * height;

                    if bar_height < 0.5 {
                        continue;
                    }

                    let frequency_factor = i as f64 / num_samples as f64;
                    let amplitude_factor = smoothed_values[i];

                    let r = 100 + (amplitude_factor * 155.0) as u8;
                    let g = 120 + ((1.0 - frequency_factor) * 100.0) as u8;
                    let b = 200 + (frequency_factor * 55.0) as u8;

                    let bar_color = Color::Rgb(r, g, b);

                    // Calculate positions for right side
                    let right_x = center_x + i as f64 * (bar_width + bar_spacing);

                    // Calculate positions for mirrored left side
                    let left_x =
                        center_x - (i as f64 + 1.0) * (bar_width + bar_spacing) + bar_spacing;

                    // Draw right side
                    ctx.draw(&Rectangle {
                        x: right_x,
                        y: 0.0,
                        width: bar_width,
                        height: bar_height,
                        color: bar_color,
                    });

                    // Draw peak for right side
                    if peak_height > 0.0 {
                        // Create a lighter version of the bar color for peaks
                        let peak_color = Color::Rgb(
                            r.saturating_add(30),
                            g.saturating_add(30),
                            b.saturating_add(30),
                        );

                        ctx.draw(&Rectangle {
                            x: right_x,
                            y: peak_height,
                            width: bar_width,
                            height: 1.0,
                            color: peak_color,
                        });
                    }

                    // Pulse effect for right side
                    let pulse = (time * 2.0 + i as f64 * 0.1).sin() * 0.3 + 0.7;
                    let pulse_height = bar_height * pulse;
                    if pulse_height > 1.0 {
                        ctx.draw(&Rectangle {
                            x: right_x,
                            y: 0.0,
                            width: bar_width,
                            height: pulse_height * 0.2,
                            color: Color::Rgb(
                                r.saturating_add(40),
                                g.saturating_add(40),
                                b.saturating_add(40),
                            ),
                        });
                    }

                    // Draw left side (mirrored)
                    ctx.draw(&Rectangle {
                        x: left_x,
                        y: 0.0,
                        width: bar_width,
                        height: bar_height,
                        color: bar_color,
                    });

                    // Draw peak for left side
                    if peak_height > 0.0 {
                        // Create a lighter version of the bar color for peaks
                        let peak_color = Color::Rgb(
                            r.saturating_add(30),
                            g.saturating_add(30),
                            b.saturating_add(30),
                        );

                        ctx.draw(&Rectangle {
                            x: left_x,
                            y: peak_height,
                            width: bar_width,
                            height: 1.0,
                            color: peak_color,
                        });
                    }

                    // Pulse effect for left side
                    if pulse_height > 1.0 {
                        ctx.draw(&Rectangle {
                            x: left_x,
                            y: 0.0,
                            width: bar_width,
                            height: pulse_height * 0.2,
                            color: Color::Rgb(
                                r.saturating_add(40),
                                g.saturating_add(40),
                                b.saturating_add(40),
                            ),
                        });
                    }
                }
            }))
    }

    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_3<'a>(
        &'a self,
        inner_area: Rect,
        spectrum: &'a [f32],
        time: f64,
    ) -> Canvas<CanvasPainter<'a>> {
        Canvas::default()
            .marker(symbols::Marker::Braille)
            .x_bounds([0.0, inner_area.width.into()])
            .y_bounds([0.0, inner_area.height.into()])
            .paint(Box::new(move |ctx| {
                let width = inner_area.width as f64;
                let height = inner_area.height as f64;

                // Fill background
                ctx.draw(&Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                    color: Color::Rgb(5, 5, 15),
                });

                // Use the full width for the bars
                let num_bars = spectrum.len().min(inner_area.width as usize);
                let bar_width = width / num_bars as f64 * 0.8;
                let bar_spacing = width / num_bars as f64 * 0.2;

                use std::sync::Mutex;
                use std::sync::OnceLock;

                static PEAKS: OnceLock<Mutex<Vec<f64>>> = OnceLock::new();
                static SMOOTHED_VALUES: OnceLock<Mutex<Vec<f64>>> = OnceLock::new();

                let peaks = PEAKS.get_or_init(|| Mutex::new(vec![0.0; num_bars]));
                let smoothed = SMOOTHED_VALUES.get_or_init(|| Mutex::new(vec![0.0; num_bars]));

                {
                    let mut smoothed_data = smoothed.lock().unwrap();
                    let mut peaks_data = peaks.lock().unwrap();

                    for (i, &value) in spectrum.iter().enumerate().take(num_bars) {
                        let alpha = 0.2;
                        let normalized_value = f64::from(value).powf(0.7);
                        smoothed_data[i] =
                            smoothed_data[i] * (1.0 - alpha) + normalized_value * alpha;

                        if smoothed_data[i] > peaks_data[i] {
                            peaks_data[i] = smoothed_data[i];
                        } else {
                            peaks_data[i] *= 0.95;
                        }
                    }
                }

                let smoothed_values = smoothed.lock().unwrap().clone();
                let peak_values = peaks.lock().unwrap().clone();

                for i in 0..num_bars {
                    let x = i as f64 * (bar_width + bar_spacing);
                    let bar_height = smoothed_values[i] * height;
                    let peak_height = peak_values[i] * height;

                    if bar_height < 0.5 {
                        continue;
                    }

                    // Create a gradient effect based on bar height and position
                    let hue = (time * 10.0 + i as f64 * 0.05) % 360.0;
                    let (r, g, b) = hsl_to_rgb(hue, 0.8, 0.5);

                    // Draw main bar
                    ctx.draw(&Rectangle {
                        x,
                        y: height - bar_height,
                        width: bar_width,
                        height: bar_height,
                        color: Color::Rgb(r, g, b),
                    });

                    // Draw peak indicator
                    if peak_height > 0.0 {
                        // Use a lighter version of the bar color for peaks
                        let peak_color = Color::Rgb(
                            r.saturating_add(50),
                            g.saturating_add(50),
                            b.saturating_add(50),
                        );

                        ctx.draw(&Rectangle {
                            x,
                            y: height - peak_height,
                            width: bar_width,
                            height: 2.0,
                            color: peak_color,
                        });
                    }

                    // Add some glow effect at the top of the bar
                    let glow_height = (bar_height * 0.2).min(5.0);
                    if glow_height > 1.0 {
                        ctx.draw(&Rectangle {
                            x,
                            y: height - bar_height,
                            width: bar_width,
                            height: glow_height,
                            color: Color::Rgb(
                                r.saturating_add(50),
                                g.saturating_add(50),
                                b.saturating_add(50),
                            ),
                        });
                    }
                }
            }))
    }

    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_4<'a>(
        &'a self,
        inner_area: Rect,
        spectrum: &'a [f32],
        time: f64,
    ) -> Canvas<CanvasPainter<'a>> {
        Canvas::default()
            .marker(symbols::Marker::Braille)
            .x_bounds([0.0, inner_area.width.into()])
            .y_bounds([0.0, inner_area.height.into()])
            .paint(Box::new(move |ctx| {
                let width = inner_area.width as f64;
                let height = inner_area.height as f64;
                let center_x = width / 2.0;

                // Fill background
                ctx.draw(&Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                    color: Color::Rgb(10, 10, 20),
                });

                // Maximize bar density - use as many samples as possible
                // Don't divide by anything to get the maximum number of bars that can fit
                let num_samples = spectrum.len().min(inner_area.width as usize);

                // Use minimal spacing between bars to maximize density
                let bar_width = (width / 2.0) / num_samples as f64 * 0.95; // Almost no gap
                let bar_spacing = (width / 2.0) / num_samples as f64 * 0.05; // Tiny spacing

                use std::sync::Mutex;
                use std::sync::OnceLock;

                // Use static data for each canvas type to prevent state confusion
                static PEAKS_4: OnceLock<Mutex<Vec<f64>>> = OnceLock::new();
                static SMOOTHED_VALUES_4: OnceLock<Mutex<Vec<f64>>> = OnceLock::new();

                let peaks = PEAKS_4.get_or_init(|| Mutex::new(vec![0.0; num_samples]));
                let smoothed = SMOOTHED_VALUES_4.get_or_init(|| Mutex::new(vec![0.0; num_samples]));

                {
                    let mut smoothed_data = smoothed.lock().unwrap();
                    let mut peaks_data = peaks.lock().unwrap();

                    // Resize if needed (in case window size changed)
                    if smoothed_data.len() != num_samples {
                        *smoothed_data = vec![0.0; num_samples];
                        *peaks_data = vec![0.0; num_samples];
                    }

                    for (i, &value) in spectrum.iter().enumerate().take(num_samples) {
                        // Apply frequency filtering - reduce high frequencies
                        let frequency_factor = i as f64 / num_samples as f64;
                        // This filter will progressively reduce values as frequency increases
                        let high_freq_filter = (1.0 - frequency_factor.powf(0.75)).max(0.15);

                        let alpha = 0.15;
                        let normalized_value = f64::from(value).powf(0.6) * high_freq_filter;
                        smoothed_data[i] =
                            smoothed_data[i] * (1.0 - alpha) + normalized_value * alpha;

                        if smoothed_data[i] > peaks_data[i] {
                            peaks_data[i] = smoothed_data[i];
                        } else {
                            peaks_data[i] *= 0.97;
                        }
                    }
                }

                let smoothed_values = smoothed.lock().unwrap().clone();
                let peak_values = peaks.lock().unwrap().clone();

                // For maximum density, use a simple skip pattern to avoid rendering every single bar
                // when the number is extremely high (which would slow down rendering)
                let skip_factor = if num_samples > 200 {
                    (num_samples / 200).max(1)
                } else {
                    1
                };

                for i in (0..num_samples).step_by(skip_factor) {
                    let bar_height = smoothed_values[i] * height;
                    let peak_height = peak_values[i] * height;

                    if bar_height < 0.3 {
                        // Lower threshold to show more bars
                        continue;
                    }

                    let frequency_factor = i as f64 / num_samples as f64;
                    let amplitude_factor = smoothed_values[i];

                    // Adjust color scheme to avoid purplish colors for high frequencies
                    // Use warmer colors (yellow to orange) instead of purplish blues
                    let r = 100 + (amplitude_factor * 155.0) as u8;
                    let g = 120 + ((1.0 - frequency_factor) * 100.0) as u8;
                    let b = 200 + (frequency_factor * 55.0) as u8;

                    let bar_color = Color::Rgb(r, g, b);

                    // Calculate positions for right side
                    let right_x = center_x + i as f64 * (bar_width + bar_spacing);

                    // Calculate positions for mirrored left side
                    let left_x =
                        center_x - (i as f64 + 1.0) * (bar_width + bar_spacing) + bar_spacing;

                    // Draw right side
                    ctx.draw(&Rectangle {
                        x: right_x,
                        y: 0.0,
                        width: bar_width,
                        height: bar_height,
                        color: bar_color,
                    });

                    // Draw peak for right side
                    if peak_height > 0.0 {
                        // Create a lighter version of the bar color for peaks
                        let peak_color = Color::Rgb(
                            r.saturating_add(30),
                            g.saturating_add(30),
                            b.saturating_add(30),
                        );

                        ctx.draw(&Rectangle {
                            x: right_x,
                            y: peak_height,
                            width: bar_width,
                            height: 1.0,
                            color: peak_color,
                        });
                    }

                    // Pulse effect for right side
                    let pulse = (time * 2.0 + i as f64 * 0.1).sin() * 0.3 + 0.7;
                    let pulse_height = bar_height * pulse;
                    if pulse_height > 1.0 {
                        ctx.draw(&Rectangle {
                            x: right_x,
                            y: 0.0,
                            width: bar_width,
                            height: pulse_height * 0.2,
                            color: Color::Rgb(
                                r.saturating_add(40),
                                g.saturating_add(40),
                                b.saturating_add(40),
                            ),
                        });
                    }

                    // Draw left side (mirrored)
                    ctx.draw(&Rectangle {
                        x: left_x,
                        y: 0.0,
                        width: bar_width,
                        height: bar_height,
                        color: bar_color,
                    });

                    // Draw peak for left side
                    if peak_height > 0.0 {
                        // Create a lighter version of the bar color for peaks
                        let peak_color = Color::Rgb(
                            r.saturating_add(30),
                            g.saturating_add(30),
                            b.saturating_add(30),
                        );

                        ctx.draw(&Rectangle {
                            x: left_x,
                            y: peak_height,
                            width: bar_width,
                            height: 1.0,
                            color: peak_color,
                        });
                    }

                    // Pulse effect for left side
                    if pulse_height > 1.0 {
                        ctx.draw(&Rectangle {
                            x: left_x,
                            y: 0.0,
                            width: bar_width,
                            height: pulse_height * 0.2,
                            color: Color::Rgb(
                                r.saturating_add(40),
                                g.saturating_add(40),
                                b.saturating_add(40),
                            ),
                        });
                    }
                }
            }))
    }
}

// Helper function to convert HSL to RGB
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn is_cava_installed() -> bool {
    Command::new("cava")
        .arg("-v")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
