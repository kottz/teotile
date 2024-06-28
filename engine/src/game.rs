use crate::error::GameError;
use core::time::Duration;

mod connect_four;
mod doodle_jump;
mod flappy_bird;
mod maze;
mod menu;
mod pong;
mod shooter;
mod snake;
mod space_invaders;
mod tetris;
mod tictactoe;
mod tugofwar;

use connect_four::ConnectFour;
use doodle_jump::DoodleJump;
use flappy_bird::FlappyBird;
use maze::{MazeGame, MazeGameMode};
use menu::Menu;
use pong::PongGame;
use shooter::MultiplayerShooter;
use snake::{SnakeGame, SnakeGameMode};
use space_invaders::SpaceInvaders;
use tetris::TetrisGame;
use tictactoe::TicTacToe;
use tugofwar::ButtonWar;

pub const GRID_SIZE: usize = 12;

pub type RenderBoard = Board<RGB, 12, 12>;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

pub trait Game {
    fn process_input(&mut self, input: GameCommand) -> Result<(), GameError>;
    fn update(&mut self, delta_time: Duration) -> Result<(), GameError>;
    fn render(&self) -> Result<RenderBoard, GameError>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    SinglePlayer,
    MultiPlayer,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Player {
    Player1,
    Player2,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Board<T, const COLS: usize, const ROWS: usize> {
    pub cells: [[T; ROWS]; COLS],
}

impl<T: Copy + Default, const COLS: usize, const ROWS: usize> Board<T, COLS, ROWS> {
    pub fn new() -> Self {
        Self {
            cells: [[T::default(); ROWS]; COLS],
        }
    }

    pub fn rows(&self) -> usize {
        ROWS
    }

    pub fn cols(&self) -> usize {
        COLS
    }

    pub fn set(&mut self, col: usize, row: usize, value: T) {
        self.cells[col][row] = value;
    }

    pub fn get(&self, col: usize, row: usize) -> T {
        self.cells[col][row]
    }
}

impl<T: Copy + Default, const COLS: usize, const ROWS: usize> Default for Board<T, COLS, ROWS> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GameEngine<T: Game = Menu> {
    game: T,
}

impl Default for GameEngine<Menu> {
    fn default() -> Self {
        Self { game: Menu::new() }
    }
}

impl<T: Game> GameEngine<T> {
    pub fn new(game: T) -> Self {
        Self { game }
    }

    pub fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        // TODO
        // Intercept input here before sending to game
        // to allow quitting the running game and returning to the main menu
        // aka if input_command.command_type == CommandType::Quit
        // then return to main menu
        //
        // This functionality is currently handled by the main menu game
        self.game.process_input(input_command)
    }

    pub fn update(&mut self, current_time: Duration) -> Result<(), GameError> {
        self.game.update(current_time)
    }

    pub fn render(&self) -> Result<RenderBoard, GameError> {
        self.game.render()
    }
}
