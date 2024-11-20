# Ratatui TUI Application Architecture Guide

## Project Structure

```
src/
├── main.rs           # Application entry point, event loop
├── app.rs            # Application container and main update logic
├── state.rs          # Application state management
├── controls/         # Input handling and state mutations
│   ├── mod.rs        # Controls module definition
│   ├── input.rs      # Input handling logic
│   └── actions.rs    # State mutation actions
├── ui/               # UI rendering logic
│   ├── mod.rs        # UI module definition
│   ├── view.rs       # Main view composition
│   └── components/   # Reusable UI components
│       ├── mod.rs    # Components module definition
│       ├── list.rs   # List component
│       ├── input.rs  # Input component
│       └── status.rs # Status bar component
└── error.rs          # Error handling

```

## Core Components Overview

### 1. State Management (`state.rs`)

```rust
#[derive(Default)]
pub struct AppState {
    // Application state fields
    pub input: String,
    pub items: Vec<String>,
    pub selected: Option<usize>,
    // Add more state as needed
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    // State query methods
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
```

### 2. Controls (`controls/`)

#### `controls/mod.rs`

```rust
pub mod input;
pub mod actions;

pub use input::*;
pub use actions::*;
```

#### `controls/actions.rs`

```rust
pub enum Action {
    Input(char),
    Submit,
    Delete,
    Select(usize),
    Quit,
}

impl Action {
    pub fn execute(&self, state: &mut AppState) -> Result<(), Error> {
        match self {
            Action::Input(c) => {
                state.input.push(*c);
                Ok(())
            },
            Action::Submit => {
                if !state.input.is_empty() {
                    state.items.push(state.input.clone());
                    state.input.clear();
                }
                Ok(())
            },
            // Add more action handlers
        }
    }
}
```

#### `controls/input.rs`

```rust
use crossterm::event::{Event, KeyCode};

pub fn handle_input(event: Event) -> Option<Action> {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Char(c) => Some(Action::Input(c)),
            KeyCode::Enter => Some(Action::Submit),
            KeyCode::Esc => Some(Action::Quit),
            // Add more key mappings
            _ => None,
        },
        _ => None,
    }
}
```

### 3. UI Components (`ui/`)

#### `ui/components/mod.rs`

```rust
pub mod list;
pub mod input;
pub mod status;

pub trait Component {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, area: Rect);
}
```

#### Example Component (`ui/components/input.rs`)

```rust
pub struct InputComponent<'a> {
    pub content: &'a str,
    pub title: &'a str,
}

impl<'a> Component for InputComponent<'a> {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let input = Paragraph::new(self.content)
            .block(Block::default()
                .title(self.title)
                .borders(Borders::ALL));
        frame.render_widget(input, area);
    }
}
```

#### `ui/view.rs`

```rust
pub struct View<'a> {
    pub state: &'a AppState,
}

impl<'a> View<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.size());

        // Render components
        InputComponent {
            content: &self.state.input,
            title: "Input",
        }.render(frame, chunks[0]);

        // Render other components
    }
}
```

### 4. Application Container (`app.rs`)

```rust
pub struct App {
    pub state: AppState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl App {
    pub fn new() -> Result<Self, Error> {
        // Terminal setup
        let terminal = setup_terminal()?;

        Ok(Self {
            state: AppState::new(),
            terminal,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            self.terminal.draw(|frame| {
                View::new(&self.state).render(frame);
            })?;

            if let Event::Key(key) = event::read()? {
                if let Some(action) = handle_input(Event::Key(key)) {
                    action.execute(&mut self.state)?;
                    if matches!(action, Action::Quit) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
```

## Best Practices and Recommendations

1. **Separation of Concerns**

   - Keep UI rendering logic completely separate from state mutations
   - Use the Component trait for all UI components
   - Handle all state changes through Actions

2. **Error Handling**

   - Create custom error types for your application
   - Use Result types consistently
   - Implement proper error propagation

3. **State Management**

   - Keep state immutable except when explicitly modified through actions
   - Use Option types for optional values
   - Implement Debug and Clone for state structures

4. **Testing**

   - Keep UI components pure and testable
   - Mock terminal for UI tests
   - Test action handlers independently

5. **Component Organization**

   - Each component should have a single responsibility
   - Use composition for complex views
   - Keep rendering logic simple and focused

6. **Input Handling**
   - Centralize input handling in the controls module
   - Map raw events to semantic actions
   - Keep input handling separate from state updates

## Future-Proofing Considerations

1. **Extensibility**

   - Use traits for components to allow easy additions
   - Keep the action system open for extension
   - Use builder patterns for complex component construction

2. **Maintenance**

   - Document component interfaces
   - Use meaningful naming conventions
   - Keep components small and focused

3. **Performance**

   - Minimize cloning where possible
   - Use references for large data structures
   - Implement custom drawing optimizations when needed

4. **State Management at Scale**
   - Consider using a proper state management solution for complex apps
   - Implement undo/redo capability if needed
   - Use event sourcing for complex state transitions
