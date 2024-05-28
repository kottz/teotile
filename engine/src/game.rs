pub mod connect_four;
use crate::ConnectFour;
use crate::RenderBoard;

use anyhow::Result;
use core::time::Duration;
pub const GRID_SIZE: usize = 12;

pub trait Game {
    fn process_input(&mut self, input: GameCommand) -> Result<()>;
    fn update(&mut self, _current_time: Duration) -> Result<()>;
    fn render(&self) -> Result<RenderBoard>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Board<T> {
    pub cells: [[T; 12]; 12],
}

impl<T: Copy> Board<T> {
    pub fn new(initial_value: T) -> Self
    where
        T: Copy,
    {
        Board {
            cells: [[initial_value; 12]; 12],
        }
    }
    pub fn size(&self) -> usize {
        self.cells.len()
    }
    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.cells[self.size() - 1 - y][x] = value;
    }
    pub fn get(&self, x: usize, y: usize) -> T {
        self.cells[self.size() - 1 - y][x]
    }
}

//type GameBoard = Board<Cell>;
//type RenderBoard = Board<RGB>;

pub struct GameBoard {
    pub cells: [[Cell; 12]; 12],
}

impl GameBoard {
    pub fn new() -> Self {
        Self {
            cells: Default::default(),
        }
    }

    pub fn size(&self) -> usize {
        self.cells.len()
    }

    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        self.cells[row][col] = cell;
    }
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[self.size() - 1 - y][x] = cell;
    }
    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[self.size() - 1 - y][x]
    }

    pub fn print(&self) {
        for row in self.cells.iter() {
            for cell in row.iter() {
                match cell {
                    Cell::Empty => {}   //print!(". "),
                    Cell::PlayerX => {} //print!("X "),
                    Cell::PlayerO => {} //print!("O "),
                }
            }
            //println!();
        }
        //println!("");
    }
    /*
        pub fn map_as_rgb(&self) -> Vec<Vec<RGB>> {
            let mut map = vec![vec![RGB::new(0, 0, 0); self.size()]; self.size()];
            for row in 0..self.size() {
                for col in 0..self.size() {
                    match self.get(col, row) {
                        Cell::Empty => map[row][col] = RGB::new(0, 0, 0),
                        Cell::PlayerX => map[row][col] = RGB::new(255, 0, 0),
                        Cell::PlayerO => map[row][col] = RGB::new(0, 0, 255),
                    }
                }
            }
            map
        }

        pub fn print_as_rgb(&self) {
            for row in self.cells.iter() {
                for cell in row.iter() {
                    match cell {
                        Cell::Empty => print!("{:?} ", RGB::new(0, 0, 0)),
                        Cell::PlayerX => print!("{:?} ", RGB::new(255, 0, 0)),
                        Cell::PlayerO => print!("{:?} ", RGB::new(0, 0, 255)),
                    }
                }
                //println!();
            }
            //println!();
        }
    */
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    PlayerX,
    PlayerO,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandType {
    Left,
    Right,
    Down,
    Up,
    Select,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameCommand {
    pub command_type: CommandType,
    pub button_state: ButtonState,
    pub player: Player,
}

impl GameCommand {
    pub fn new(command_type: CommandType, button_state: ButtonState, player: Player) -> Self {
        Self {
            command_type,
            button_state,
            player,
        }
    }
}

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct PlayerId(usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Player {
    Player1,
    Player2,
}

// TODO: Should the gameengine itself act as mainmenu?
// or should it be a separate struct?
// need some logic that will allow us to cancel the current game

pub struct GameEngine<T: Game = ConnectFour> {
    game: T,
}

impl Default for GameEngine<ConnectFour> {
    fn default() -> Self {
        Self {
            game: ConnectFour::new(),
        }
    }
}

impl<T: Game> GameEngine<T> {
    pub fn new(game: T) -> Self {
        Self { game }
    }

    pub fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
        // TODO
        // Intercept input here before sending to game
        // to allow quitting the running game and returning to the main menu
        // aka if input_command.command_type == CommandType::Quit
        // then return to main menu
        self.game.process_input(input_command)
    }

    pub fn update(&mut self, current_time: Duration) -> Result<()> {
        self.game.update(current_time)
    }

    pub fn render(&self) -> Result<RenderBoard> {
        self.game.render()
    }
}
