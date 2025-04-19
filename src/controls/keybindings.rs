use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Represents an action that can be performed in the application
#[derive(Debug, Clone)]
pub struct Action {
    pub name: String,
    pub description: String,
}

/// Manages all keybindings for the application
#[derive(Debug)]
pub struct Keybindings {
    pub bindings: HashMap<KeyEvent, Action>,
}

impl Keybindings {
    /// Create a new keybindings map with all defaults
    pub fn new() -> Self {
        let mut bindings = HashMap::new();

        // Playback controls
        bindings.insert(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            Action {
                name: "play_selected".to_string(),
                description: "Play selected track".to_string(),
            },
        );

        bindings.insert(
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
            Action {
                name: "toggle_playback".to_string(),
                description: "Toggle play/pause".to_string(),
            },
        );

        bindings.insert(
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
            Action {
                name: "stop".to_string(),
                description: "Stop playback".to_string(),
            },
        );

        // Navigation
        bindings.insert(
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            Action {
                name: "select_next".to_string(),
                description: "Select next track".to_string(),
            },
        );

        bindings.insert(
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            Action {
                name: "select_previous".to_string(),
                description: "Select previous track".to_string(),
            },
        );

        // Volume/Balance controls
        bindings.insert(
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            Action {
                name: "volume_down".to_string(),
                description: "Decrease volume".to_string(),
            },
        );

        bindings.insert(
            KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            Action {
                name: "volume_up".to_string(),
                description: "Increase volume".to_string(),
            },
        );

        bindings.insert(
            KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT),
            Action {
                name: "balance_left".to_string(),
                description: "Adjust balance left".to_string(),
            },
        );

        bindings.insert(
            KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT),
            Action {
                name: "balance_right".to_string(),
                description: "Adjust balance right".to_string(),
            },
        );

        // Bass/Treble controls
        bindings.insert(
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            Action {
                name: "bass_up".to_string(),
                description: "Increase bass".to_string(),
            },
        );

        bindings.insert(
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            Action {
                name: "bass_down".to_string(),
                description: "Decrease bass".to_string(),
            },
        );

        bindings.insert(
            KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT),
            Action {
                name: "treble_up".to_string(),
                description: "Increase treble".to_string(),
            },
        );

        bindings.insert(
            KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT),
            Action {
                name: "treble_down".to_string(),
                description: "Decrease treble".to_string(),
            },
        );

        // Application controls
        bindings.insert(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            Action {
                name: "quit".to_string(),
                description: "Quit application".to_string(),
            },
        );

        // Visualizer canvas controls
        for i in 0..=9 {
            bindings.insert(
                KeyEvent::new(
                    KeyCode::Char(char::from_digit(i, 10).unwrap()),
                    KeyModifiers::NONE,
                ),
                Action {
                    name: format!("visualizer_mode_{}", i),
                    description: format!("Select visualizer mode {}", i),
                },
            );
        }

        // Help overlay
        bindings.insert(
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
            Action {
                name: "toggle_help".to_string(),
                description: "Show/hide help".to_string(),
            },
        );

        Self { bindings }
    }

    /// Get an action by key event if it exists
    pub fn get_action(&self, key: &KeyEvent) -> Option<&Action> {
        self.bindings.get(key)
    }

    /// Get all keybindings
    pub fn get_all_bindings(&self) -> &HashMap<KeyEvent, Action> {
        &self.bindings
    }

    /// Get a pretty string representation of a key event
    pub fn key_event_to_string(key: &KeyEvent) -> String {
        let modifier_str = match key.modifiers {
            KeyModifiers::SHIFT => "Shift+",
            KeyModifiers::CONTROL => "Ctrl+",
            KeyModifiers::ALT => "Alt+",
            _ => "",
        };

        let key_str = match key.code {
            KeyCode::Char(' ') => "Space".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            _ => format!("{:?}", key.code),
        };

        format!("{}{}", modifier_str, key_str)
    }
}
