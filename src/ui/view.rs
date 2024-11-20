use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;

use super::components::{
    music_library_ui::MusicLibraryUI, playback_control_ui::PlaybackControlUI,
    sound_control_ui::SoundControlUI, visualizer_ui::VisualizerUI,
};

pub struct UIManager {
    music_library: MusicLibraryUI,
    sound_control: SoundControlUI,
    playback_controls: PlaybackControlUI,
    visualizer: VisualizerUI,
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            music_library: MusicLibraryUI::new(),
            visualizer: VisualizerUI::new(),
            sound_control: SoundControlUI::new(),
            playback_controls: PlaybackControlUI::new(),
        }
    }

    pub fn render(&self, frame: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(2, 10), Constraint::Ratio(8, 10)])
            .split(frame.area());

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(2, 3), // Visualizer
                Constraint::Ratio(1, 3), // Controls
            ])
            .split(chunks[1]);

        let control_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_layout[1]);

        self.music_library.render(frame, chunks[0], app.get_state());
        self.visualizer
            .render(frame, main_layout[0], app.get_state());
        self.sound_control
            .render(frame, control_chunks[0], app.get_state());
        self.playback_controls
            .render(frame, control_chunks[1], app.get_state());
    }
}
