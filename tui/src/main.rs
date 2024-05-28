use std::collections::HashMap;
use std::io::{self, stdout};
use std::time::Duration;
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
    // for _ in 0..10 {
    //     let command = GameCommand::new(CommandType::Right, ButtonState::Pressed, Player::Player1);
    //     let _ = engine.process_input(command);
    //     let ten_millis = Duration::from_millis(10);
    //     let output = engine.update(ten_millis);
    //     let board = engine.render();
    //     println!("{:?}", board);
    // }
    //engine.process_input(GameCommand::Up);
    let mut app = App::new();
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        //terminal.draw(ui(&app))?;
        terminal.draw(|f| ui(f, &app))?;
        //should_quit = handle_events()?;
        //let command: GameCommand = handle_events()?;
        if let Ok(Some(command)) = handle_events() {
            if command.command_type == CommandType::Quit {
                should_quit = true;
            }
            let _ = engine.process_input(command);
        }
        app.grid.update_grid_from_renderboard(&engine.render().unwrap());
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

// fn handle_events() -> io::Result<bool> {
//     if event::poll(std::time::Duration::from_millis(50))? {
//         if let Event::Key(key) = event::read()? {
//             if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
//                 return Ok(true);
//             }
//             if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('w') {
//                 println!("W pressed");
//             }
//         }
//     }
//     Ok(false)
// }
fn handle_events() -> io::Result<Option<GameCommand>> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            let button_state = if key.kind == event::KeyEventKind::Press {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            };

            let command = match key.code {
                KeyCode::Char('q') => return Ok(Some(GameCommand::new(CommandType::Quit, button_state, Player::Player1))),
                KeyCode::Char('w') => GameCommand::new(CommandType::Up, button_state, Player::Player1),
                KeyCode::Char('a') => GameCommand::new(CommandType::Left, button_state, Player::Player1),
                KeyCode::Char('s') => GameCommand::new(CommandType::Down, button_state, Player::Player1),
                KeyCode::Char('d') => GameCommand::new(CommandType::Right, button_state, Player::Player1),
                KeyCode::Char('e') => GameCommand::new(CommandType::Select, button_state, Player::Player1),
                KeyCode::Char('r') => GameCommand::new(CommandType::Select, button_state, Player::Player1),
                KeyCode::Char('f') => GameCommand::new(CommandType::Quit, button_state, Player::Player1),
                KeyCode::Up => GameCommand::new(CommandType::Up, button_state, Player::Player2),
                KeyCode::Left => GameCommand::new(CommandType::Left, button_state, Player::Player2),
                KeyCode::Down => GameCommand::new(CommandType::Down, button_state, Player::Player2),
                KeyCode::Right => GameCommand::new(CommandType::Right, button_state, Player::Player2),
                KeyCode::Enter => GameCommand::new(CommandType::Select, button_state, Player::Player2),
                KeyCode::Char('m') => GameCommand::new(CommandType::Select, button_state, Player::Player2),
                KeyCode::Backspace => GameCommand::new(CommandType::Quit, button_state, Player::Player2),
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
    //frame.render_widget(app.grid.grid_canvas(), grid_space);

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
                let grid_size = GRID_SIZE as u8;
                for i in 0..=grid_size - 1 {
                    for j in 0..=grid_size - 1 {
                        let color = self.coords[i as usize][j as usize];
                        ctx.draw(&Points {
                            coords: &[(i as f64, j as f64)],
                            color: color, //Color::Rgb(150 + (j + i) * 4, 150 + j * 4, 150 + i * 4),
                        })
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
    // fn draw_grid_from_renderboard(&self, render_board: &RenderBoard) -> impl Widget + '_ {
    //     let render_clone = render_board.clone();
    //     //let color: RGB = render_board.get(0,0);
    //     Canvas::default()
    //         .block(Block::bordered())
    //         .marker(self.marker)
    //         .paint(move |ctx| {
    //             let grid_size = GRID_SIZE as u8;
    //             for i in 0..=grid_size - 1 {
    //                 for j in 0..=grid_size - 1 {
    //                     let color: RGB = render_clone.get(i as usize, j as usize);
    //                     ctx.draw(&Points {
    //                         coords: &[(i as f64, j as f64)],
    //                         color: Color::Rgb(color.r, color.g, color.b),
    //                     })
    //                 }
    //             }
    //         })
    //         .x_bounds([0.0, 11.0])
    //         .y_bounds([0.0, 11.0])
    // }
}

struct App {
    x: f64,
    y: f64,
    grid: Grid,
}

impl App {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            grid: Grid::new(),
        }
    }
}

struct PlayerInput {
    name: String,
    buttons: HashMap<String, bool>,
}

impl PlayerInput {
    fn new() -> Self {
        let name = "Player".to_string();
        let mut buttons = HashMap::new();
        buttons.insert("up".to_string(), false);
        buttons.insert("down".to_string(), false);
        buttons.insert("left".to_string(), false);
        buttons.insert("right".to_string(), false);
        buttons.insert("a".to_string(), false);
        buttons.insert("b".to_string(), false);
        buttons.insert("start".to_string(), false);
        buttons.insert("select".to_string(), false);
        Self { name, buttons }
    }

    fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    fn create_widget(&self) -> impl Widget + '_ {
        //let mut state = ListState::default();
        let mut button_vec = self
            .buttons
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>();
        button_vec.sort();
        let list = List::new(button_vec)
            .block(Block::bordered().title(self.name.as_str()))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);
        return list;
    }
}
