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

    pub fn render(&self, frame: &mut Frame, area: Rect, spectrum: &[f32]) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Spectrum Analyzer");

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let spectrum_data = spectrum;

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

//#[derive(Default, Clone, Debug)]
//pub struct Visualizer {
//    smooth: Vec<f32>,
//}
//
//impl Visualizer {
//    pub fn new((width, height): (usize, usize)) -> Self {
//        Self {
//            smooth: vec![0.0; 1024],
//        }
//    }
//
//    pub fn update_spectrum(&mut self, frame: &[f32]) {
//        for (i, value) in self.smooth.iter_mut().enumerate() {
//            let mut new_value: f32 = 0.0;
//            let p = i as f32 * 0.25;
//            let r = 4.0;
//            let range = (p - r).clamp(0.0, frame.len() as f32) as usize
//                ..(p + r).clamp(0.0, frame.len() as f32) as usize;
//
//            for q in range {
//                let d = p - q as f32;
//                new_value +=
//                    (self.height as f32 / 16.0) * (1.0 + q as f32 / 8.0) * frame[q] / (1.0 + d * d);
//            }
//
//            *value += (new_value.sqrt() - *value) * 0.2;
//        }
//    }
//
//    pub fn render(&self, frame: &mut Frame, area: Rect) {
//        let min = self.smooth.iter().copied().fold(f32::MAX, f32::min);
//
//        // Calculate the drawing dimensions based on the provided area
//        let width = area.width as usize;
//        let height = area.height as usize;
//
//        // Create points for the spectrum
//        let points: Vec<(String, (u16, u16))> = (0..width / 2)
//            .flat_map(|x| {
//                let mut points = Vec::new();
//
//                // Upper half of spectrum
//                let y =
//                    height as f32 - 2.0 - (self.smooth[x] - min).clamp(0.0, (height - 2) as f32);
//                points.push(("•".to_string(), (x as u16, y as u16)));
//                points.push(("•".to_string(), ((width - 1 - x) as u16, y as u16)));
//
//                // Lower half of spectrum
//                let y = (self.smooth[width / 2 + x] - min).clamp(0.0, (height - 2) as f32);
//                points.push(("•".to_string(), (x as u16, y as u16)));
//                points.push(("•".to_string(), ((width - 1 - x) as u16, y as u16)));
//
//                points
//            })
//            .collect();
//
//        // Draw each point using Canvas
//        for (symbol, (x, y)) in points {
//            // Ensure the point is within the area bounds
//            if x < area.width && y < area.height {
//                let buffer = frame.buffer_mut();
//                buffer[(area.left() + x, area.top() + y)].set_symbol(&symbol);
//            }
//        }
//    }
//}
