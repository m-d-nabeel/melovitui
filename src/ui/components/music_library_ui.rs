use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::sync::{Arc, Mutex};

use crate::controls::music_library::MusicLibrary;

pub struct MusicLibraryUI {
    style: MusicLibraryStyle,
}

struct MusicLibraryStyle {
    title_color: Color,
    number_color: Color,
    filename_color: Color,
    extension_color: Color,
    highlight_bg_color: Color,
    selected_bg_color: Color,
    selected_fg_color: Color,
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
            selected_bg_color: Color::LightGreen,
            selected_fg_color: Color::Black,
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

    pub fn render(&self, frame: &mut Frame, area: Rect, lib_state: Arc<Mutex<MusicLibrary>>) {
        let lib_state = lib_state.lock().unwrap();
        let block = Block::default()
            .title("Music Library")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(self.style.title_color));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let tracks = &lib_state.tracks;
        let current_track_idx = lib_state.current_index;
        let selected_track_idx = lib_state.selected_index;

        log::info!("current_track_idx: {:?}", current_track_idx);
        log::info!("selected_track_idx: {:?}", current_track_idx);

        let items: Vec<ListItem> = tracks
            .iter()
            .enumerate()
            .map(|(i, track)| {
                // Track number styling
                let number = Span::styled(
                    format!("{:2}.", i + 1),
                    Style::default().fg(self.style.number_color),
                );

                // Filename styling with enhanced selection and current track handling
                let file_name = if i == current_track_idx {
                    Span::styled(
                        &track.title,
                        Style::default()
                            .fg(self.style.selected_fg_color)
                            .bg(self.style.selected_bg_color)
                            .add_modifier(Modifier::BOLD),
                    )
                } else if Some(i) == selected_track_idx {
                    Span::styled(
                        &track.title,
                        Style::default()
                            .fg(self.style.highlight_bg_color)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(&track.title, Style::default().fg(self.style.filename_color))
                };

                // Extension styling
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

        let list = List::new(items)
            .style(Style::default())
            .highlight_symbol("> ") // Optional: add a selector
            .highlight_style(
                Style::default()
                    .bg(self.style.highlight_bg_color)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(list, inner);

        // Stats rendering remains the same
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
