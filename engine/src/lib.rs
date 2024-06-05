#![no_std]

mod game;
mod animation;
pub use game::RenderBoard;
pub use game::RGB;
pub use game::{
    Board, ButtonState, Cell, CommandType, Game, GameCommand, GameEngine, Player, GRID_SIZE,
};
