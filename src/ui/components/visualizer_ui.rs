use core::f64;
use std::f64::consts::PI;

use ratatui::layout::Alignment;
use ratatui::widgets::canvas::Circle;
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
            1 => self.get_canvas_1(inner_area, &spectrum, time),
            2 => self.get_canvas_2(inner_area, &spectrum, time),
            3 => self.get_canvas_3(inner_area, &spectrum, time),
            4 => self.get_canvas_4(inner_area, &spectrum, time),
            5 => self.get_canvas_5(inner_area, &spectrum, time),
            6 => self.get_canvas_6(inner_area, &spectrum, time),
            7 => self.get_canvas_7(inner_area, &spectrum, time),
            8 => self.get_canvas_8(inner_area, &spectrum, time),
            9 => self.get_canvas_9(inner_area, &spectrum, time),
            // Use CAVA as the default visualizer
            _ => self.get_canvas_cava(inner_area, &spectrum, time),
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

    #[allow(dead_code, elided_named_lifetimes)]
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
}
impl VisualizerUI {
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
                // Create circular visualizer
                let center_x = inner_area.width as f64 / 2.0;
                let center_y = inner_area.height as f64 / 2.0;
                let max_radius = (inner_area.width.min(inner_area.height) as f64 / 2.0) * 0.8;

                // Draw rotating circles
                for (i, &value) in spectrum.iter().enumerate().take(32) {
                    let angle = (i as f64 / 32.0) * 2.0 * std::f64::consts::PI + time;
                    let radius = max_radius * (0.3 + value as f64 * 0.7);

                    // Create spiral effect
                    for j in 0..5 {
                        let spiral_factor = j as f64 * 0.2;
                        let spiral_radius = radius * (1.0 - spiral_factor);
                        let x = center_x + spiral_radius * angle.cos();
                        let y = center_y + spiral_radius * angle.sin();

                        let alpha = (1.0 - spiral_factor) * 0.8;
                        let color = match self.style.bar_color {
                            Color::Rgb(r, g, b) => Color::Rgb(
                                (r as f64 * alpha) as u8,
                                (g as f64 * alpha) as u8,
                                (b as f64 * alpha) as u8,
                            ),
                            _ => self.style.bar_color,
                        };

                        ctx.draw(&Circle {
                            x,
                            y,
                            radius: 1.0 + value as f64 * 2.0,
                            color,
                        });
                    }
                }

                // Add orbital particles
                for i in 0..16 {
                    let particle_angle =
                        time * 2.0 + (i as f64 / 16.0) * 2.0 * std::f64::consts::PI;
                    let orbit_radius = max_radius * 0.5;
                    let x = center_x + orbit_radius * particle_angle.cos();
                    let y = center_y + orbit_radius * particle_angle.sin();

                    ctx.draw(&Circle {
                        x,
                        y,
                        radius: 0.5,
                        color: self.style.particle_color,
                    });
                }
            }))
    }

    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_5<'a>(
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
                // Create DNA helix visualization
                let helix_width = inner_area.width as f64;
                let helix_height = inner_area.height as f64;
                let frequency = 2.0;
                let amplitude = helix_height * 0.2;

                for i in 0..64 {
                    let x = (i as f64 / 64.0) * helix_width;
                    let phase =
                        time * 2.0 + (x / helix_width) * 2.0 * std::f64::consts::PI * frequency;

                    // Create two intertwining helixes
                    let y1 = helix_height / 2.0 + amplitude * phase.sin();
                    let y2 = helix_height / 2.0 + amplitude * (phase + std::f64::consts::PI).sin();

                    // Add spectrum-based variations
                    let spectrum_value = spectrum[i.min(spectrum.len() - 1)] as f64;
                    let bar_height = spectrum_value * amplitude;

                    // Draw connecting bars with gradient
                    let steps = 8;
                    for step in 0..steps {
                        let t = step as f64 / (steps - 1) as f64;
                        let y = y1 * (1.0 - t) + y2 * t;

                        let alpha = (1.0 - (t - 0.5).abs() * 2.0) * 0.8;
                        let color = match self.style.bar_color {
                            Color::Rgb(r, g, b) => Color::Rgb(
                                (r as f64 * alpha) as u8,
                                (g as f64 * alpha) as u8,
                                (b as f64 * alpha) as u8,
                            ),
                            _ => self.style.bar_color,
                        };

                        ctx.draw(&Rectangle {
                            x,
                            y: y - bar_height / 2.0,
                            width: 1.0,
                            height: bar_height,
                            color,
                        });
                    }

                    // Add particles along the helixes
                    if spectrum_value > 0.3 {
                        ctx.draw(&Circle {
                            x,
                            y: y1,
                            radius: 0.5 + spectrum_value,
                            color: self.style.particle_color,
                        });
                        ctx.draw(&Circle {
                            x,
                            y: y2,
                            radius: 0.5 + spectrum_value,
                            color: self.style.peak_color,
                        });
                    }
                }
            }))
    }

    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_6<'a>(
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
                let center_x = inner_area.width as f64 / 2.0;
                let center_y = inner_area.height as f64 / 2.0;

                // Create kaleidoscope effect
                let num_segments = 8;
                let segment_angle = 2.0 * std::f64::consts::PI / num_segments as f64;

                for (i, &value) in spectrum.iter().enumerate().take(32) {
                    let base_angle = time + i as f64 * 0.2;
                    let radius = value as f64 * inner_area.height as f64 * 0.4;

                    // Create symmetrical pattern
                    for segment in 0..num_segments {
                        let angle = base_angle + segment as f64 * segment_angle;
                        let x = center_x + radius * angle.cos();
                        let y = center_y + radius * angle.sin();

                        // Draw trailing effect
                        for trail in 0..5 {
                            let trail_radius = radius * (1.0 - trail as f64 * 0.2);
                            let trail_x = center_x + trail_radius * angle.cos();
                            let trail_y = center_y + trail_radius * angle.sin();

                            let alpha = 1.0 - trail as f64 * 0.2;
                            let color = match self.style.bar_color {
                                Color::Rgb(r, g, b) => Color::Rgb(
                                    (r as f64 * alpha) as u8,
                                    (g as f64 * alpha) as u8,
                                    (b as f64 * alpha) as u8,
                                ),
                                _ => self.style.bar_color,
                            };

                            // Draw connecting lines
                            if trail > 0 {
                                ctx.draw(&Line {
                                    x1: x,
                                    y1: y,
                                    x2: trail_x,
                                    y2: trail_y,
                                    color,
                                });
                            }

                            // Add particles at vertices
                            if value > 0.2 {
                                ctx.draw(&Circle {
                                    x: trail_x,
                                    y: trail_y,
                                    radius: 0.5 + value as f64,
                                    color: self.style.particle_color,
                                });
                            }
                        }
                    }
                }

                // Add center starburst
                let burst_rays = 16;
                for ray in 0..burst_rays {
                    let ray_angle = (ray as f64 / burst_rays as f64) * 2.0 * std::f64::consts::PI;
                    let ray_length = (time * 2.0).sin() * 10.0;

                    ctx.draw(&Line {
                        x1: center_x,
                        y1: center_y,
                        x2: center_x + ray_length * ray_angle.cos(),
                        y2: center_y + ray_length * ray_angle.sin(),
                        color: self.style.peak_color,
                    });
                }
            }))
    }
}

impl VisualizerUI {
    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_7<'a>(
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
                if spectrum.is_empty() {
                    return; // Avoid rendering if spectrum is empty
                }

                let center_x = inner_area.width as f64 / 2.0;
                let center_y = inner_area.height as f64 / 2.0;
                let max_radius = (inner_area.width.min(inner_area.height) as f64 / 2.0) * 0.8;

                let golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
                let num_points = 50;

                for i in 0..num_points {
                    let t = i as f64 / num_points as f64;
                    let theta = 2.0 * PI * golden_ratio * i as f64 + time;
                    let radius = max_radius * t;

                    let x = (center_x + radius * theta.cos()).clamp(0.0, inner_area.width as f64);
                    let y = (center_y + radius * theta.sin()).clamp(0.0, inner_area.height as f64);

                    // Safe spectrum indexing
                    let spectrum_idx = ((i as f32 / num_points as f32)
                        * (spectrum.len() - 1) as f32)
                        .round() as usize;
                    let value = spectrum.get(spectrum_idx).unwrap_or(&0.0);

                    let particle_size = (0.5 + *value as f64 * 3.0).min(5.0);
                    let phase = time * 2.0 + t * 4.0 * PI;

                    for ring in 0..3 {
                        let ring_radius =
                            particle_size * (1.0 + ring as f64 * 0.3 + phase.sin() * 0.2);
                        let alpha = (1.0 - ring as f64 * 0.3).max(0.0) * 0.8;

                        let color = match self.style.bar_color {
                            Color::Rgb(r, g, b) => Color::Rgb(
                                (r as f64 * alpha).min(255.0) as u8,
                                (g as f64 * alpha).min(255.0) as u8,
                                (b as f64 * alpha).min(255.0) as u8,
                            ),
                            _ => self.style.bar_color,
                        };

                        ctx.draw(&Circle {
                            x,
                            y,
                            radius: ring_radius.max(0.1),
                            color,
                        });
                    }

                    if i > 0 {
                        let prev_t = (i - 1) as f64 / num_points as f64;
                        let prev_theta = 2.0 * PI * golden_ratio * (i - 1) as f64 + time;
                        let prev_radius = max_radius * prev_t;

                        let prev_x = (center_x + prev_radius * prev_theta.cos())
                            .clamp(0.0, inner_area.width as f64);
                        let prev_y = (center_y + prev_radius * prev_theta.sin())
                            .clamp(0.0, inner_area.height as f64);

                        let line_alpha = (*value as f64 * 0.5).min(1.0);
                        let line_color = match self.style.wave_color {
                            Color::Rgb(r, g, b) => Color::Rgb(
                                (r as f64 * line_alpha).min(255.0) as u8,
                                (g as f64 * line_alpha).min(255.0) as u8,
                                (b as f64 * line_alpha).min(255.0) as u8,
                            ),
                            _ => self.style.wave_color,
                        };

                        ctx.draw(&Line {
                            x1: prev_x,
                            y1: prev_y,
                            x2: x,
                            y2: y,
                            color: line_color,
                        });
                    }
                }
            }))
    }

    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_8<'a>(
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
                if spectrum.is_empty() {
                    return; // Avoid rendering if spectrum is empty
                }

                let width = inner_area.width as f64;
                let height = inner_area.height as f64;

                let field_resolution = 10;
                let cell_width = width / field_resolution as f64;
                let cell_height = height / field_resolution as f64;

                for i in 0..field_resolution {
                    for j in 0..field_resolution {
                        let x = i as f64 * cell_width;
                        let y = j as f64 * cell_height;

                        let angle = (x * 0.05 + time) * (y * 0.05 + time).sin() * PI;

                        let spectrum_idx = ((i * field_resolution + j) as f32
                            / (field_resolution * field_resolution) as f32
                            * spectrum.len() as f32)
                            .round() as usize;
                        let value = spectrum.get(spectrum_idx).unwrap_or(&0.0);

                        let length = cell_width * *value as f64;
                        let end_x = x + length * angle.cos();
                        let end_y = y + length * angle.sin();

                        let steps = 5;
                        for step in 0..steps {
                            let t = step as f64 / steps as f64;
                            let alpha = (1.0 - t) * *value as f64;

                            let color = match self.style.bar_color {
                                Color::Rgb(r, g, b) => Color::Rgb(
                                    (r as f64 * alpha).min(255.0) as u8,
                                    (g as f64 * alpha).min(255.0) as u8,
                                    (b as f64 * alpha).min(255.0) as u8,
                                ),
                                _ => self.style.bar_color,
                            };

                            let step_x = x + (end_x - x) * t;
                            let step_y = y + (end_y - y) * t;

                            ctx.draw(&Circle {
                                x: step_x,
                                y: step_y,
                                radius: 0.5 + *value as f64 * (1.0 - t),
                                color,
                            });
                        }
                    }
                }
            }))
    }

    #[allow(dead_code, elided_named_lifetimes)]
    fn get_canvas_9<'a>(
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
                if spectrum.is_empty() {
                    return; // Avoid rendering if spectrum is empty
                }

                let center_x = inner_area.width as f64 / 2.0;
                let center_y = inner_area.height as f64 / 2.0;
                let max_radius = (inner_area.width.min(inner_area.height) as f64 / 2.0) * 0.8;

                // Create Mandala pattern
                let num_rings = 5;
                let points_per_ring = 12;

                for ring in 0..num_rings {
                    let ring_radius = max_radius * (ring + 1) as f64 / num_rings as f64;
                    let ring_phase = time * (1.0 + ring as f64 * 0.5);

                    // Draw main ring structure
                    for point in 0..points_per_ring {
                        let angle = (point as f64 / points_per_ring as f64) * 2.0 * PI + ring_phase;
                        let _x = center_x + ring_radius * angle.cos();
                        let _y = center_y + ring_radius * angle.sin();

                        // Get spectrum value for this point
                        let spectrum_idx = ((ring * points_per_ring + point) as f32
                            / (num_rings * points_per_ring) as f32
                            * spectrum.len() as f32)
                            as usize;
                        let value = spectrum[spectrum_idx.min(spectrum.len() - 1)];

                        // Create lotus petal effect
                        let petal_points = 8;
                        for p in 0..petal_points {
                            let petal_t = p as f64 / petal_points as f64;
                            let petal_angle = angle + petal_t * PI / 6.0;
                            let petal_radius = ring_radius * (0.8 + 0.2 * (petal_t * PI).sin());

                            let petal_x = center_x + petal_radius * petal_angle.cos();
                            let petal_y = center_y + petal_radius * petal_angle.sin();

                            let alpha = value as f64 * (1.0 - petal_t * 0.5);
                            let color = match self.style.bar_color {
                                Color::Rgb(r, g, b) => Color::Rgb(
                                    (r as f64 * alpha) as u8,
                                    (g as f64 * alpha) as u8,
                                    (b as f64 * alpha) as u8,
                                ),
                                _ => self.style.bar_color,
                            };

                            // Draw petal segments
                            if p > 0 {
                                let prev_t = (p - 1) as f64 / petal_points as f64;
                                let prev_angle = angle + prev_t * PI / 6.0;
                                let prev_radius = ring_radius * (0.8 + 0.2 * (prev_t * PI).sin());

                                let prev_x = center_x + prev_radius * prev_angle.cos();
                                let prev_y = center_y + prev_radius * prev_angle.sin();

                                ctx.draw(&Line {
                                    x1: prev_x,
                                    y1: prev_y,
                                    x2: petal_x,
                                    y2: petal_y,
                                    color,
                                });
                            }

                            // Add glowing particles at petal tips
                            if value > 0.3 && p == petal_points - 1 {
                                for glow in 0..3 {
                                    let glow_alpha = (1.0 - glow as f64 * 0.3) * value as f64;
                                    let glow_color = match self.style.peak_color {
                                        Color::Rgb(r, g, b) => Color::Rgb(
                                            (r as f64 * glow_alpha) as u8,
                                            (g as f64 * glow_alpha) as u8,
                                            (b as f64 * glow_alpha) as u8,
                                        ),
                                        _ => self.style.peak_color,
                                    };

                                    ctx.draw(&Circle {
                                        x: petal_x,
                                        y: petal_y,
                                        radius: 0.5 + glow as f64 * 0.5,
                                        color: glow_color,
                                    });
                                }
                            }
                        }
                    }
                }

                // Add central energy burst
                let burst_radius = (time * 3.0).sin() * max_radius * 0.2;
                for i in 0..16 {
                    let burst_angle = (i as f64 / 16.0) * 2.0 * PI;
                    let burst_x = center_x + burst_radius * burst_angle.cos();
                    let burst_y = center_y + burst_radius * burst_angle.sin();

                    ctx.draw(&Line {
                        x1: center_x,
                        y1: center_y,
                        x2: burst_x,
                        y2: burst_y,
                        color: self.style.wave_color,
                    });
                }
            }))
    }

    // This function will use CAVA's raw output to create a visualizer
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
