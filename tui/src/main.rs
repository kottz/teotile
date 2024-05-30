use std::collections::HashMap;
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
    frame.render_widget(Block::bordered().title("Player Input"), inner_layout[1]);
    let player_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(inner_layout[1]);

    let grid_space = create_grid_layout(&inner_layout[0]);
    frame.render_widget(app.grid.grid_canvas(), grid_space);

    for (i, pl) in player_layout.iter().enumerate() {
        let player_string = format!("Player {}", i + 1);
        let player_input = PlayerInput::new().name(player_string.as_str());
        frame.render_widget(player_input.create_widget(), *pl);
    }
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

const BUTTONS: [(&str, bool); 8] = [
    ("up", false),
    ("down", false),
    ("left", false),
    ("right", false),
    ("a", false),
    ("b", false),
    ("start", false),
    ("select", false),
];

struct PlayerInput {
    name: String,
    buttons: HashMap<String, bool>,
}

impl PlayerInput {
    fn new() -> Self {
        let name = "Player".to_string();
        let buttons = BUTTONS
            .iter()
            .cloned()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        Self { name, buttons }
    }

    fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    fn create_widget(&self) -> impl Widget + '_ {
        let button_map = self.buttons.iter().map(|(k, v)| format!("{}: {}", k, v));
        let mut button_vec = button_map.collect::<Vec<String>>();
        button_vec.sort();
        List::new(button_vec)
            .block(Block::bordered().title(self.name.as_str()))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
    }
}
