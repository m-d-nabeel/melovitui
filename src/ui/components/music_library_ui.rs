use std::sync::{Arc, Mutex};

use crate::state::AppState;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct MusicLibraryUI {
    style: MusicLibraryStyle,
}

struct MusicLibraryStyle {
    title_color: Color,
    number_color: Color,
    filename_color: Color,
    extension_color: Color,
    highlight_bg_color: Color,
    stats_bg_color: Color,
    stats_fg_color: Color,
}

impl Default for MusicLibraryStyle {
    fn default() -> Self {
        Self {
            title_color: Color::Cyan,
            number_color: Color::Yellow,
            filename_color: Color::White,
            extension_color: Color::DarkGray,
            highlight_bg_color: Color::DarkGray,
            stats_bg_color: Color::Cyan,
            stats_fg_color: Color::Black,
        }
    }
}

impl MusicLibraryUI {
    pub fn new() -> Self {
        Self {
            style: MusicLibraryStyle::default(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, app_state: Arc<Mutex<AppState>>) {
        let app_state = app_state.lock().unwrap();

        let block = Block::default()
            .title("Music Library")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(self.style.title_color));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let tracks = &app_state.library.tracks;
        let items: Vec<ListItem> = tracks
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let number = Span::styled(
                    format!("{:1}.", i + 1),
                    Style::default().fg(self.style.number_color),
                );
                let file_name =
                    Span::styled(&track.title, Style::default().fg(self.style.filename_color));
                let extension = Span::styled(
                    format!(
                        " [{}]",
                        track
                            .path
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or("")
                    ),
                    Style::default()
                        .fg(self.style.extension_color)
                        .add_modifier(Modifier::ITALIC),
                );
                ListItem::new(Line::from(vec![number, file_name, extension]))
            })
            .collect();

        let list = List::new(items).style(Style::default()).highlight_style(
            Style::default()
                .bg(self.style.highlight_bg_color)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_widget(list, inner);

        if !tracks.is_empty() {
            let stats = format!(" {} tracks ", tracks.len());
            let stats_style = Style::default()
                .fg(self.style.stats_fg_color)
                .bg(self.style.stats_bg_color);

            let stats_text = Line::from(vec![Span::styled(stats, stats_style)]);
            let stats_area = Rect {
                x: area.x + 1,
                y: area.bottom() - 1,
                width: area.width - 2,
                height: 1,
            };

            frame.render_widget(
                Paragraph::new(stats_text).alignment(Alignment::Right),
                stats_area,
            );
        }
    }
}
