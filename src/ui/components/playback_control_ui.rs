use std::sync::{Arc, Mutex};

use crate::state::AppState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, LineGauge, Paragraph},
    Frame,
};

pub struct PlaybackControlUI {
    style: PlaybackControlStyle,
}

struct PlaybackControlStyle {
    text_color: Color,
    timeline_color: Color,
    timeline_bg_color: Color,
    button_color: Color,
    active_button_color: Color,
}

impl Default for PlaybackControlStyle {
    fn default() -> Self {
        Self {
            text_color: Color::White,
            timeline_color: Color::Cyan,
            timeline_bg_color: Color::DarkGray,
            button_color: Color::Gray,
            active_button_color: Color::Green,
        }
    }
}

impl PlaybackControlUI {
    pub fn new() -> Self {
        Self {
            style: PlaybackControlStyle::default(),
        }
    }

    fn format_duration(duration: std::time::Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, app_state: Arc<Mutex<AppState>>) {
        let app_state = app_state.lock().unwrap();

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Playback Controls");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .margin(1)
            .split(inner);

        // Determine current track title
        let song_text = app_state
            .playback
            .current_track
            .map(|idx| app_state.library.tracks[idx].title.clone())
            .map(|track| format!("▶ {}", track))
            .unwrap_or_else(|| "No song playing".to_string());

        frame.render_widget(
            Paragraph::new(song_text)
                .style(Style::default().fg(self.style.text_color))
                .alignment(Alignment::Center),
            chunks[0],
        );

        let timeline_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(7),
                Constraint::Min(0),
                Constraint::Length(7),
            ])
            .split(chunks[1]);

        frame.render_widget(
            Paragraph::new(Self::format_duration(app_state.playback.elapsed))
                .style(Style::default().fg(self.style.text_color))
                .alignment(Alignment::Left),
            timeline_layout[0],
        );

        let progress = if app_state.playback.duration.as_secs() > 0 {
            app_state.playback.elapsed.as_secs_f64() / app_state.playback.duration.as_secs_f64()
        } else {
            0.0
        };

        frame.render_widget(
            LineGauge::default()
                .filled_style(Style::default().fg(self.style.timeline_color))
                .ratio(progress)
                .line_set(symbols::line::THICK)
                .label(""),
            timeline_layout[1],
        );

        frame.render_widget(
            Paragraph::new(Self::format_duration(app_state.playback.duration))
                .style(Style::default().fg(self.style.text_color))
                .alignment(Alignment::Right),
            timeline_layout[2],
        );
    }
}
