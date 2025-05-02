use core::f64;

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
            peak_color: Color::Rgb(255, 215, 0),
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
            1 => self.get_canvas_3(inner_area, &spectrum, time),
            // Use CAVA as the default visualizer
            _ => self.get_canvas_cava(inner_area, &spectrum, time),
        };

        // Render the selected canvas
        frame.render_widget(canvas, inner_area);
    }
}

impl VisualizerUI {
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
                let center_y = inner_area.height as f64 / 2.0;
                let max_height = inner_area.height as f64 / 3.0;

                // Draw centerline with wave effect
                for x in 0..inner_area.width {
                    let wave = (x as f64 * 0.1 + time).sin() * 2.0;
                    ctx.draw(&Line {
                        x1: x as f64,
                        y1: center_y + wave - 0.5,
                        x2: x as f64,
                        y2: center_y + wave + 0.5,
                        color: Color::Rgb(30, 30, 40),
                    });
                }

                // Mirror effect visualization
                for (i, &value) in spectrum
                    .iter()
                    .enumerate()
                    .take(inner_area.width as usize / 2)
                {
                    let x = i as f64;
                    let mirror_x = inner_area.width as f64 - i as f64 - 1.0;
                    let height = value as f64 * max_height;

                    // Upper wave
                    let upper_wave = (time * 3.0 + i as f64 * 0.1).sin() * 1.0;
                    let upper_y = center_y - height + upper_wave;

                    // Lower wave (inverted phase)
                    let lower_wave =
                        (time * 3.0 + i as f64 * 0.1 + std::f64::consts::PI).sin() * 1.0;
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

                    // Add particles at high intensity points
                    if value > 0.5 {
                        let particle_color = self.style.particle_color;
                        let particle_y = upper_y + (time * 5.0 + i as f64).sin() * 3.0;

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
                        ctx.draw(&Rectangle {
                            x,
                            y: peak_y,
                            width: bar_width - 1.0,
                            height: 1.0,
                            color: self.style.peak_color,
                        });
                    }
                }
            }))
    }
}
