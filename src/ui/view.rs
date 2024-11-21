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

        let library_state = app.get_library_state();
        let visualizer_state = app.get_visualizer_state();
        let sound_state = app.get_sound_state();
        let playback_state = app.get_playback_state();

        // Determine current track title
        let playback_state_unarced = playback_state.lock().unwrap();
        let library_state_unarced = library_state.lock().unwrap();
        let song_text = playback_state_unarced
            .current_track
            .map(|idx| library_state_unarced.tracks[idx].title.clone())
            .map(|track| format!("▶ {}", track))
            .unwrap_or_else(|| "No song playing".to_string());

        drop(playback_state_unarced);
        drop(library_state_unarced);

        self.music_library.render(frame, chunks[0], library_state);
        self.visualizer
            .render(frame, main_layout[0], visualizer_state);
        self.sound_control
            .render(frame, control_chunks[0], sound_state);
        self.playback_controls
            .render(frame, control_chunks[1], playback_state, song_text);
    }
}
