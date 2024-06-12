#![no_std]
include!(concat!(env!("OUT_DIR"), "/pixel_art.rs"));
mod animation;
mod game;
pub use game::RenderBoard;
pub use game::RGB;
pub use game::{Board, ButtonState, CommandType, Game, GameCommand, GameEngine, Player, GRID_SIZE};
