#![no_std]
include!(concat!(env!("OUT_DIR"), "/pixel_art.rs"));
mod animation;
mod error;
mod game;
mod random;
pub use error::GameError;
pub use game::RGB;
pub use game::RenderBoard;
pub use game::{Board, ButtonState, CommandType, GRID_SIZE, Game, GameCommand, GameEngine, Player};
