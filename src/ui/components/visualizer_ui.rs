use ratatui::layout::Alignment;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{
        block::Title,
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
    pub bar_color: Color,
    pub peak_color: Color,
    pub grid_color: Color,
    pub particle_color: Color,
    pub wave_color: Color,
}

impl Default for VisualizerStyle {
    fn default() -> Self {
        Self {
            block_style: Style::default()
                .bg(Color::Black)
                .fg(Color::Rgb(150, 160, 255)),
            bar_color: Color::Rgb(65, 105, 225),
            peak_color: Color::Rgb(255, 215, 0),
            grid_color: Color::Gray,
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

    pub fn render(&self, frame: &mut Frame, area: Rect, spectrum: &[f32]) {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .title(Title::from("Spectrum Visualizer "))
            .style(self.style.block_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let canvas = self.get_canvas_3(inner_area, spectrum, time);
        frame.render_widget(canvas, inner_area);
    }
}

impl VisualizerUI {
    #[allow(dead_code)]
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
                let bar_width = inner_area.width as f32 / 64.0;

                // Draw background grid
                for x in (0..inner_area.width).step_by(8) {
                    ctx.draw(&Line {
                        x1: x as f64,
                        y1: 0.0,
                        x2: x as f64,
                        y2: inner_area.height as f64,
                        color: match self.style.grid_color {
                            Color::Rgb(r, g, b) => Color::Rgb(r, g, b / 5),
                            _ => self.style.grid_color,
                        },
                    });
                }

                // Draw spectrum bars with effects
                for (i, &value) in spectrum.iter().enumerate().take(64) {
                    let x = i as f32 * bar_width;
                    let normalized_value = value * inner_area.height as f32;

                    // Main bar with gradient effect
                    let gradient_steps = 8;
                    let step_height = normalized_value / gradient_steps as f32;
                    for step in 0..gradient_steps {
                        let alpha_factor = 1.0 - (step as f32 / gradient_steps as f32) * 0.7;
                        let color = match self.style.bar_color {
                            Color::Rgb(r, g, b) => Color::Rgb(
                                (r as f32 * alpha_factor) as u8,
                                (g as f32 * alpha_factor) as u8,
                                (b as f32 * alpha_factor) as u8,
                            ),
                            _ => self.style.bar_color,
                        };
                        ctx.draw(&Rectangle {
                            x: x.into(),
                            y: (step as f32 * step_height).into(),
                            width: (bar_width * 0.9).into(),
                            height: step_height.into(),
                            color,
                        });
                    }

                    // Glowing peak line
                    let peak_y = normalized_value - 1.0;
                    for glow in 1..=3 {
                        let alpha_factor = 0.3 / glow as f32;
                        let color = match self.style.peak_color {
                            Color::Rgb(r, g, b) => Color::Rgb(
                                (r as f32 * alpha_factor) as u8,
                                (g as f32 * alpha_factor) as u8,
                                (b as f32 * alpha_factor) as u8,
                            ),
                            _ => self.style.peak_color,
                        };
                        ctx.draw(&Rectangle {
                            x: x.into(),
                            y: (peak_y - glow as f32).into(),
                            width: (bar_width * 0.9).into(),
                            height: (2.0 + glow as f64 * 2.0),
                            color,
                        });
                    }

                    // Add reflection effect
                    let reflection_height = normalized_value * 0.3;
                    for reflect_step in 0..5 {
                        let alpha_factor = 0.2 - (reflect_step as f32 * 0.04);
                        let color = match self.style.bar_color {
                            Color::Rgb(r, g, b) => Color::Rgb(
                                (r as f32 * alpha_factor) as u8,
                                (g as f32 * alpha_factor) as u8,
                                (b as f32 * alpha_factor) as u8,
                            ),
                            _ => self.style.bar_color,
                        };
                        ctx.draw(&Rectangle {
                            x: x.into(),
                            y: (inner_area.height as f32 - reflection_height + reflect_step as f32)
                                .into(),
                            width: (bar_width * 0.9).into(),
                            height: (reflection_height / 5.0).into(),
                            color,
                        });
                    }

                    // Add dancing particles
                    let particle_offset = (time * 5.0 + i as f64 * 0.5).sin() * 5.0;
                    if value > 0.3 {
                        ctx.draw(&Rectangle {
                            x: (x + bar_width * 0.3).into(),
                            y: (normalized_value + particle_offset as f32).into(),
                            width: (bar_width * 0.3).into(),
                            height: 1.0,
                            color: self.style.particle_color,
                        });
                    }
                }

                // Add wave effect
                let wave_points: Vec<(f64, f64)> = (0..inner_area.width)
                    .map(|x| {
                        let wave = (x as f64 * 0.1 + time * 2.0).sin() * 3.0;
                        (x as f64, (inner_area.height as f64 - 5.0) + wave)
                    })
                    .collect();

                for points in wave_points.windows(2) {
                    if let [p1, p2] = points {
                        let color = match self.style.wave_color {
                            Color::Rgb(r, g, b) => Color::Rgb(r / 3, g / 3, b / 3),
                            _ => self.style.wave_color,
                        };
                        ctx.draw(&Line {
                            x1: p1.0,
                            y1: p1.1,
                            x2: p2.0,
                            y2: p2.1,
                            color,
                        });
                    }
                }
            }))
    }

    #[allow(dead_code)]
    fn get_canvas_2<'a>(
        &'a self,
        inner_area: Rect,
        spectrum: &'a [f32],
        time: f64,
    ) -> Canvas<CanvasPainter<'a>> {
        Canvas::default()
            .marker(symbols::Marker::Dot)
            .x_bounds([0.0, inner_area.width.into()])
            .y_bounds([0.0, inner_area.height.into()])
            .paint(Box::new(move |ctx| {
                let bar_width = inner_area.width as f32 / 64.0;

                // Draw background grid for depth effect
                for x in (0..inner_area.width).step_by(4) {
                    ctx.draw(&Line {
                        x1: x as f64,
                        y1: 0.0,
                        x2: x as f64,
                        y2: inner_area.height as f64,
                        color: Color::Rgb(20, 20, 30),
                    });
                }

                for (i, &value) in spectrum.iter().enumerate().take(64) {
                    let x = i as f32 * bar_width;
                    let normalized_value = value * inner_area.height as f32;

                    // Main bar with 3D effect
                    ctx.draw(&Rectangle {
                        x: x.into(),
                        y: 0.0,
                        width: (bar_width * 0.8).into(),
                        height: normalized_value.into(),
                        color: self.style.bar_color,
                    });

                    // Side shadow for 3D effect
                    ctx.draw(&Rectangle {
                        x: (x + bar_width * 0.8).into(),
                        y: 0.0,
                        width: (bar_width * 0.2).into(),
                        height: normalized_value.into(),
                        color: match self.style.bar_color {
                            Color::Rgb(r, g, b) => Color::Rgb(r / 2, g / 2, b / 2),
                            _ => self.style.bar_color,
                        },
                    });

                    // Animated peak line
                    let peak_bounce = (time * 4.0 + i as f64 * 0.2).sin() * 2.0;
                    ctx.draw(&Rectangle {
                        x: x.into(),
                        y: (normalized_value - 1.0) as f64 + peak_bounce,
                        width: (bar_width * 0.8).into(),
                        height: 2.0,
                        color: self.style.peak_color,
                    });

                    // Glow effect around peak
                    ctx.draw(&Rectangle {
                        x: x.into(),
                        y: (normalized_value - 2.0) as f64 + peak_bounce,
                        width: (bar_width * 0.8).into(),
                        height: 4.0,
                        color: match self.style.peak_color {
                            Color::Rgb(r, g, b) => Color::Rgb(r / 3, g / 3, b / 3),
                            _ => self.style.peak_color,
                        },
                    });
                }
            }))
    }
    #[allow(dead_code)]
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
}
