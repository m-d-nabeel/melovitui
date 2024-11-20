use std::sync::{Arc, Mutex};

use crate::state::AppState;
use ratatui::{
    layout::Rect,
    style::Color,
    symbols,
    widgets::{
        canvas::{Canvas, Rectangle},
        Block, Borders,
    },
    Frame,
};

pub struct VisualizerUI {
    style: VisualizerStyle,
}

pub struct VisualizerStyle {
    bar_color: Color,
    peak_color: Color,
    background_color: Color,
}

impl Default for VisualizerStyle {
    fn default() -> Self {
        Self {
            bar_color: Color::Cyan,
            peak_color: Color::Red,
            background_color: Color::Black,
        }
    }
}

impl VisualizerUI {
    pub fn new() -> Self {
        Self {
            style: VisualizerStyle::default(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, app_state: Arc<Mutex<AppState>>) {
        let app_state = app_state.lock().unwrap();

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Spectrum Analyzer");

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let spectrum_data = &app_state.audio.spectrum_data;

        let canvas = Canvas::default()
            .marker(symbols::Marker::Dot)
            .x_bounds([0.0, inner_area.width.into()])
            .y_bounds([0.0, inner_area.height.into()])
            .paint(|ctx| {
                let bar_width = inner_area.width as f32 / 64.0;
                for (i, &value) in spectrum_data.iter().enumerate().take(64) {
                    let x = i as f32 * bar_width;
                    let normalized_value = value * inner_area.height as f32;

                    ctx.draw(&Rectangle {
                        x: x.into(),
                        y: 0.0,
                        width: (bar_width * 0.8).into(),
                        height: normalized_value.into(),
                        color: self.style.bar_color,
                    });

                    // Add peak line at the top of each bar
                    ctx.draw(&Rectangle {
                        x: x.into(),
                        y: (normalized_value - 1.0).into(),
                        width: (bar_width * 0.8).into(),
                        height: 1.0,
                        color: self.style.peak_color,
                    });
                }
            });

        frame.render_widget(canvas, inner_area);
    }
}
