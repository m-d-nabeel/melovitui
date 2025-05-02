use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::{app::App, controls::playback_state::PlaybackStatus};

use super::components::{
    help_ui::HelpUI, music_library_ui::MusicLibraryUI, playback_control_ui::PlaybackControlUI,
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

    pub fn render(&mut self, frame: &mut Frame, app: &App) {
        // Base layout
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

        let (library_state, sound_state, playback_state) = (
            app.get_library_state(),
            app.get_sound_state(),
            app.get_playback_state(),
        );

        let audio_system = app.get_audio_system();
        let audio_system = audio_system.lock();
        let spectrum = audio_system.get_current_frame();
        let canvas_type = audio_system.get_visualizer_canvas_type();

        let song_text = {
            let playback_state = playback_state.lock();
            let library_state = library_state.lock();
            if let Some(idx) = playback_state.current_track {
                let track_title = library_state.tracks[idx].title.clone();

                match playback_state.status {
                    PlaybackStatus::Stopped => format!("⏹ {}", track_title), // Stop button
                    PlaybackStatus::Playing => format!("⏸ {}", track_title), // Play button
                    PlaybackStatus::Paused => format!("▶ {}", track_title),  // Pause button
                }
            } else {
                "No song playing".to_string()
            }
        };

        // Render main UI components
        self.music_library.render(frame, chunks[0], library_state);
        self.visualizer
            .render(frame, main_layout[0], spectrum, canvas_type);
        self.sound_control
            .render(frame, control_chunks[0], sound_state);
        self.playback_controls
            .render(frame, control_chunks[1], playback_state, song_text);

        if app.show_help {
            // takes whole frame as a board for render
            // makes the bg dim and creates an overlay
            // no need to intantiate an object for this
            HelpUI::render(frame, app.get_keybindings());
        }
    }
}
