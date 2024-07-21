use std::io::{self, stdout};
use std::time::Instant;
use teotile::{ButtonState, CommandType, GameCommand, GameEngine, Player, RenderBoard, RGB};
const GRID_SIZE: usize = 12;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut engine = GameEngine::default();
    let mut prev_instant = Instant::now();
    let mut app = App::new();
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| ui(f, &app))?;
        match handle_events() {
            Ok(Some(command)) => {
                let _ = engine.process_input(command);
            }
            Ok(None) => {}
            Err(_) => {
                should_quit = true;
            }
        }
        let current_instant = Instant::now();
        let delta = current_instant - prev_instant;
        prev_instant = current_instant;
        let _ = engine.update(delta);
        app.grid
            .update_grid_from_renderboard(&engine.render().unwrap());
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<Option<GameCommand>> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            let button_state = if key.kind == event::KeyEventKind::Press {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            };

            let command = match key.code {
                KeyCode::Char('q') | KeyCode::Char('f') => {
                    return Ok(Some(GameCommand::new(
                        CommandType::Quit,
                        button_state,
                        Player::Player1,
                    )))
                }
                KeyCode::Char('w') => {
                    GameCommand::new(CommandType::Up, button_state, Player::Player1)
                }
                KeyCode::Char('a') => {
                    GameCommand::new(CommandType::Left, button_state, Player::Player1)
                }
                KeyCode::Char('s') => {
                    GameCommand::new(CommandType::Down, button_state, Player::Player1)
                }
                KeyCode::Char('d') => {
                    GameCommand::new(CommandType::Right, button_state, Player::Player1)
                }
                KeyCode::Char('e') | KeyCode::Char('r') => {
                    GameCommand::new(CommandType::Select, button_state, Player::Player1)
                }
                KeyCode::Up => GameCommand::new(CommandType::Up, button_state, Player::Player2),
                KeyCode::Left => GameCommand::new(CommandType::Left, button_state, Player::Player2),
                KeyCode::Down => GameCommand::new(CommandType::Down, button_state, Player::Player2),
                KeyCode::Right => {
                    GameCommand::new(CommandType::Right, button_state, Player::Player2)
                }
                KeyCode::Enter | KeyCode::Char('m') => {
                    GameCommand::new(CommandType::Select, button_state, Player::Player2)
                }
                KeyCode::Backspace => {
                    GameCommand::new(CommandType::Quit, button_state, Player::Player2)
                }
                KeyCode::Esc | KeyCode::Char('u') => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Escape key pressed, quitting",
                    ))
                }
                _ => return Ok(None),
            };

            return Ok(Some(command));
        }
    }
    Ok(None)
}

fn ui(frame: &mut Frame, app: &App) {
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ],
    )
    .split(frame.size());
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Title Bar"),
        main_layout[0],
    );
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Status Bar"),
        main_layout[2],
    );

    let inner_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(main_layout[1]);
    frame.render_widget(Block::bordered().title("Grid"), inner_layout[0]);
    frame.render_widget(Block::bordered().title("Player Controls"), inner_layout[1]);
    let player_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(inner_layout[1]);

    let grid_space = create_grid_layout(&inner_layout[0]);
    frame.render_widget(app.grid.grid_canvas(), grid_space);

    let player1_input = PlayerInput::new()
        .name("Player 1")
        .controls(&PLAYER1_CONTROLS);
    let player2_input = PlayerInput::new()
        .name("Player 2")
        .controls(&PLAYER2_CONTROLS);

    frame.render_widget(player1_input.create_widget(), player_layout[0]);
    frame.render_widget(player2_input.create_widget(), player_layout[1]);
}

fn create_grid_layout(input_area: &Rect) -> Rect {
    let grid_container_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(GRID_SIZE as u16 + 2),
            Constraint::Min(1),
        ])
        .split(*input_area);

    let grid_space_horizontal = grid_container_layout[1];

    let grid_container_layout_vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(GRID_SIZE as u16 + 2),
            Constraint::Min(1),
        ])
        .split(grid_space_horizontal);

    grid_container_layout_vertical[1]
}

struct Grid {
    coords: [[Color; GRID_SIZE]; GRID_SIZE],
    marker: Marker,
}

impl Grid {
    fn new() -> Self {
        Self {
            coords: [[Color::Rgb(0, 0, 0); GRID_SIZE]; GRID_SIZE],
            marker: Marker::Dot,
        }
    }

    fn grid_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered())
            .marker(self.marker)
            .paint(|ctx| {
                for (i, row) in self.coords.iter().enumerate() {
                    for (j, &color) in row.iter().enumerate() {
                        ctx.draw(&Points {
                            coords: &[(i as f64, j as f64)],
                            color,
                        });
                    }
                }
            })
            .x_bounds([0.0, 11.0])
            .y_bounds([0.0, 11.0])
    }
    fn update_grid_from_renderboard(&mut self, render_board: &RenderBoard) {
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                let color: RGB = render_board.get(i, j);
                self.coords[i][j] = Color::Rgb(color.r, color.g, color.b);
            }
        }
    }
}

struct App {
    grid: Grid,
}

impl App {
    fn new() -> Self {
        Self { grid: Grid::new() }
    }
}

const PLAYER1_CONTROLS: [(&str, &str); 6] = [
    ("Move Up", "W"),
    ("Move Down", "S"),
    ("Move Left", "A"),
    ("Move Right", "D"),
    ("Select", "E/R"),
    ("Quit", "Q/F"),
];

const PLAYER2_CONTROLS: [(&str, &str); 6] = [
    ("Move Up", "↑"),
    ("Move Down", "↓"),
    ("Move Left", "←"),
    ("Move Right", "→"),
    ("Select", "Enter/M"),
    ("Quit", "Backspace"),
];

struct PlayerInput {
    name: String,
    controls: Vec<(String, String)>,
}

impl PlayerInput {
    fn new() -> Self {
        Self {
            name: "Player".to_string(),
            controls: Vec::new(),
        }
    }

    fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    fn controls(mut self, controls: &[(&str, &str)]) -> Self {
        self.controls = controls
            .iter()
            .map(|&(action, key)| (action.to_string(), key.to_string()))
            .collect();
        self
    }

    fn create_widget(&self) -> impl Widget + '_ {
        let max_action_length = self
            .controls
            .iter()
            .map(|(action, _)| action.len())
            .max()
            .unwrap_or(0);

        let controls = self.controls.iter().map(|(action, key)| {
            let padded_line = format!(
                "{:<width$} {}",
                format!("{}:", action),
                key,
                width = max_action_length + 1
            );
            ListItem::new(padded_line)
        });

        List::new(controls)
            .block(Block::bordered().title(self.name.as_str()))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ")
    }
}
