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

type RenderBoard = Board<RGB>;

fn game_loop() -> Result<()> {
    let text_in = TextInput {};
    let con_out = ColorOutput {};
    let mut game = ConnectFour::new(text_in, con_out);

    loop {
        game.process_input()?;
        game.update()?;
        game.render()?;

        if let ConnectFourState::Win(_) = game.state {
            let text_in = TextInput {};
            let con_out = ColorOutput {};
            game = ConnectFour::new(text_in, con_out);
        }
    }
    //Ok(())
}

fn main() -> Result<()> {
    game_loop()?;

    #[cfg(feature = "std")]
    let input = ConsoleInput {};
    #[cfg(feature = "std")]
    let output = ConsoleOutput {};

    #[cfg(not(feature = "std"))]
    let input = TextInput {};
    #[cfg(not(feature = "std"))]
    let output = ColorOutput {};

    let mut game = ConnectFour::new(input, output);
    let win = game.check_win((0, 0), 4);
    //println!("win: {:?}", win);
    for i in 0..5 {
        game.make_move(i, Cell::PlayerX).unwrap();
        game.make_move(i, Cell::PlayerO).unwrap();
    }
    //game.set(11,11, Cell::PlayerO);
    game.make_move(0, Cell::PlayerX).unwrap();
    game.make_move(0, Cell::PlayerO).unwrap();
    game.make_move(0, Cell::PlayerX).unwrap();
    game.make_move(1, Cell::PlayerO).unwrap();
    game.make_move(2, Cell::PlayerX).unwrap();
    game.make_move(1, Cell::PlayerO).unwrap();
    game.make_move(3, Cell::PlayerX).unwrap();
    game.make_move(1, Cell::PlayerO).unwrap();
    game.make_move(2, Cell::PlayerX).unwrap();
    game.make_move(2, Cell::PlayerO).unwrap();
    game.make_move(2, Cell::PlayerX).unwrap();
    game.make_move(2, Cell::PlayerO).unwrap();
    let win = game.check_win((0, 0), 5);
    //println!("win: {:?}", win);
    /*
        game.make_move(0, Cell::PlayerX).unwrap();
        game.make_move(0, Cell::PlayerO).unwrap();
        game.make_move(0, Cell::PlayerX).unwrap();
        game.make_move(2, Cell::PlayerO).unwrap();
        game.make_move(1, Cell::PlayerX).unwrap();
        game.make_move(8, Cell::PlayerO).unwrap();
        game.make_move(1, Cell::PlayerX).unwrap();
    */
    game.board.print();

    //test this
    let con_in = ConsoleInput {};
    let con_out = ConsoleOutput {};
    //let gameboard = GameBoard::new();
    //let mut game = ConnectFour { input: con_in, output: con_out, board: gameboard, active_player: Cell::PlayerX };
    let mut game = ConnectFour::new(con_in, con_out);
    let _ = game.make_move(2, Cell::PlayerX);
    let _ = game.make_move(3, Cell::PlayerX);
    let _ = game.make_move(0, Cell::PlayerX);
    let _ = game.make_move(1, Cell::PlayerX);
    game.board.print();
    let win = game.check_win((3, 0), 2);
    //println!("win: {:?}", win);

    Ok(())
}
