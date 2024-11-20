use std::sync::{Arc, Mutex};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::AppState;

use super::audio_gauge::AudioGauge;

pub struct ControlStyle {
    knob_color: Color,
    active_color: Color,
    text_color: Color,
}

pub struct SoundControlUI {
    style: ControlStyle,
}

#[derive(Debug, Clone, Copy)]
pub enum AudioControlType {
    Volume,
    Bass,
    Treble,
    Balance,
}

impl Default for ControlStyle {
    fn default() -> Self {
        Self {
            knob_color: Color::White,
            active_color: Color::Green,
            text_color: Color::Gray,
        }
    }
}

impl SoundControlUI {
    pub fn new() -> Self {
        Self {
            style: ControlStyle::default(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, app_state: Arc<Mutex<AppState>>) {
        let app_state = app_state.lock().unwrap();

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Audio Controls");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(inner);

        let controls = [
            (AudioControlType::Volume, app_state.audio.volume / 100.0),
            (AudioControlType::Bass, app_state.audio.bass / 100.0),
            (AudioControlType::Treble, app_state.audio.treble / 100.0),
            (
                AudioControlType::Balance,
                (app_state.audio.balance + 100.0) / 200.0,
            ),
        ];

        for (i, (control_type, value)) in controls.iter().enumerate() {
            self.render_gauge(frame, chunks[i], &control_type.to_string(), *value);
        }
    }

    fn render_gauge(&self, frame: &mut Frame, area: Rect, label: &str, value: f32) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new(label).style(Style::default().fg(self.style.text_color)),
            layout[0],
        );

        let value_text = format!("{:3}%", (value * 100.0) as u8);
        let audio_control_widget = AudioGauge::new(value, value_text.to_string())
            .style(Style::default().fg(self.style.knob_color))
            .show_percentage(false);

        frame.render_widget(audio_control_widget, layout[1]);
    }
}

impl std::fmt::Display for AudioControlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioControlType::Volume => write!(f, "Volume"),
            AudioControlType::Bass => write!(f, "Bass"),
            AudioControlType::Treble => write!(f, "Treble"),
            AudioControlType::Balance => write!(f, "Balance"),
        }
    }
}
