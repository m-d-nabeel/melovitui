use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::controls::keybindings::Keybindings;

pub struct HelpUI;

impl HelpUI {
    pub fn render(frame: &mut Frame, keybindings: &Keybindings) {
        // Dimmed full-screen backdrop
        let backdrop = Block::default().style(Style::default().add_modifier(Modifier::DIM));
        frame.render_widget(backdrop, frame.area());

        // Centered help overlay
        let help_area = Self::centered_rect(70, 70, frame.area());

        let block = Block::default()
            .title(" Keyboard Shortcuts ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(help_area);
        frame.render_widget(Clear, help_area);
        frame.render_widget(block, help_area);

        // 1) Pull out (key, description) tuples…
        let mut bindings: Vec<(String, String)> = keybindings
            .get_all_bindings()
            .iter()
            .map(|(key, action)| {
                let key_str = Keybindings::key_event_to_string(key);
                (key_str, action.description.clone())
            })
            .collect();

        // 2) Sort *before* building Rows
        bindings.sort_by(|a, b| a.0.cmp(&b.0));

        // 3) Convert sorted tuples into Rows with styled cells
        let row_style_even = Style::default();
        let row_style_odd = Style::default().bg(Color::Rgb(35, 37, 45));
        let rows: Vec<Row> = bindings
            .into_iter()
            .enumerate()
            .map(|(i, (key_str, desc))| {
                let key_cell = Cell::from(key_str).style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                );
                let desc_cell = Cell::from(desc);

                let row_style = if i % 2 == 0 {
                    row_style_even
                } else {
                    row_style_odd
                };

                Row::new(vec![key_cell, desc_cell]).style(row_style)
            })
            .collect();

        // Build the table with both rows and column widths
        let table = Table::new(
            rows,
            [Constraint::Percentage(25), Constraint::Percentage(75)],
        )
        .header(
            Row::new(vec![
                Cell::from("Key").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Action").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
            // .style(Style::default().bg(Color::DarkGray))
            .bottom_margin(1)
            .height(2),
        )
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .widths([Constraint::Percentage(25), Constraint::Percentage(75)]);

        frame.render_widget(table, inner);

        // Add hint at the bottom of the screen
        let hint_text = "Press any key to close help";
        let hint_area = Rect::new(
            (frame.area().width - hint_text.len() as u16) / 2,
            frame.area().height - 2,
            hint_text.len() as u16,
            1,
        );
        let hint = Paragraph::new(hint_text).style(
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        );
        frame.render_widget(Clear, hint_area);
        frame.render_widget(hint, hint_area);
    }

    /// Helper function to create a centered rect using a percentage of the available rect
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
