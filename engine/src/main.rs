#![cfg_attr(not(feature = "std"), no_std)]

mod io;
use io::{ColorOutput, ConsoleInput, ConsoleOutput, Input, TextInput};

mod game;
use game::connect_four::{ConnectFour, ConnectFourState};
use game::{Board, Cell, Game, GameBoard, GameCommand, GRID_SIZE};

use anyhow::Result;

// Update the render functions to make
// use of the states. Makes it more cleaner to move
// the game into the winning state and then match
// on that in the render function
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
//reexport GameEngine



type RenderBoard = Board<RGB>;

// fn game_loop() -> Result<()> {
//     let text_in = TextInput {};
//     let con_out = ColorOutput {};
//     let mut game = ConnectFour::new(text_in, con_out);
//
//     loop {
//         game.process_input()?;
//         game.update()?;
//         game.render()?;
//
//         if let ConnectFourState::Win(_) = game.state {
//             let text_in = TextInput {};
//             let con_out = ColorOutput {};
//             game = ConnectFour::new(text_in, con_out);
//         }
//     }
//     //Ok(())
// }

// fn main() -> Result<()> {
//     game_loop()?;
//     Ok(())
// }
