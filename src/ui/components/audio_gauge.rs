use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct AudioGauge {
    value: f32,
    label: String,
    style: Style,
    show_percentage: bool,
    gradient: bool,
}

impl AudioGauge {
    pub fn new(value: f32, label: String) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            label,
            style: Style::default(),
            show_percentage: true,
            gradient: true,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    #[allow(dead_code)]
    pub fn gradient(mut self, enabled: bool) -> Self {
        self.gradient = enabled;
        self
    }

    fn get_gradient_color(&self, position: f32) -> Color {
        if position < 0.3 {
            Color::Rgb(50, 150, 255) // Light Blue
        } else if position < 0.6 {
            Color::Rgb(0, 100, 255) // Medium Blue
        } else if position < 0.8 {
            Color::Rgb(0, 70, 200) // Dark Blue
        } else {
            Color::Rgb(0, 50, 150) // Very Dark Blue
        }
    }
}

impl Widget for AudioGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 1 {
            return;
        }

        let gauge_start = self.label.len() as u16 + 5;
        let available_width = area.width.saturating_sub(gauge_start);
        let usable_width = if self.show_percentage {
            available_width.saturating_sub(6) // Space for percentage
        } else {
            available_width
        } - 5;

        // Render label
        buf.set_string(area.x, area.y, &self.label, self.style);

        // Render gauge
        let filled_count = ((usable_width as f32 * self.value) as u16).max(1);
        for i in 0..usable_width {
            let x = area.x + gauge_start + i;
            let position = i as f32 / usable_width as f32;
            let cell = &mut buf[(x, area.y)];
            if i < filled_count {
                if self.gradient {
                    let style = self.style.fg(self.get_gradient_color(position));
                    cell.set_char(get_level_char(position)).set_style(style);
                } else {
                    cell.set_char('▐').set_style(self.style.fg(Color::Blue));
                }
            } else {
                cell.set_char('░').set_style(self.style.fg(Color::DarkGray));
            };
        }

        // Render percentage if enabled
        if self.show_percentage {
            let percentage = format!("{:>3}%", (self.value * 100.0) as u32);
            buf.set_string(area.x + area.width - 4, area.y, &percentage, self.style);
        }

        // Add visual markers
        if area.height > 1 {
            let marker_style = Style::default().fg(Color::DarkGray);
            for (pos, marker) in [(0.0, "0"), (0.5, "50"), (1.0, "100")] {
                let x = area.x + gauge_start + (usable_width as f32 * pos) as u16;
                if x + marker.len() as u16 <= area.x + area.width {
                    buf.set_string(x, area.y + 1, marker, marker_style);
                }
            }
        }
    }
}

fn get_level_char(position: f32) -> char {
    if position < 0.2 {
        '▏'
    } else if position < 0.4 {
        '▎'
    } else if position < 0.6 {
        '▍'
    } else if position < 0.8 {
        '▌'
    } else {
        '▋'
    }
}
