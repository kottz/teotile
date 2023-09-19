//#![no_std]
#![cfg_attr(not(feature = "std"), no_std)]

mod game;
use game::{Game, Board, GameCommand, GameBoard, GRID_SIZE, Cell};
use game::connect_four::{ConnectFour, ConnectFourState};

use anyhow::Result;

use smallvec::{SmallVec, smallvec};

use rand::Rng;

use crossterm::{
    cursor::MoveLeft,
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};

#[cfg(feature = "std")]
use std::io::{stdout, Write};


pub trait Input {
    fn read(&mut self) -> Option<GameCommand>;
}

pub trait Output {
    fn write(&self, render_board: &RenderBoard) -> Result<()>;
}

pub struct ConsoleInput {
    //buffer: String,
}

impl Input for ConsoleInput {
    fn read(&mut self) -> Option<GameCommand> {
        let event = read().unwrap();

        //////println!("Event::{:?}\r", event);

        if event == Event::Key(KeyCode::Char('q').into()) {
            //println!("quit");
            return Some(GameCommand::Quit);
        }
        if event == Event::Key(KeyCode::Char('w').into()) {
            //println!("up");
            return Some(GameCommand::Up);
        }

        /*
        match event {
            Event::Key(KeyCode::Char('q')) => Some(GameCommand::Quit),
            Event::Key(KeyCode::Char('a')) => Some(GameCommand::Left),
            Event::Key(KeyCode::Char('d')) => Some(GameCommand::Right),
            Event::Key(KeyCode::Char('s')) => Some(GameCommand::Down),
            Event::Key(KeyCode::Char('w')) => Some(GameCommand::Up),
            Event::Key(KeyCode::Enter) => Some(GameCommand::Select),
            _ => None,
        }*/

        todo!()
    }
}

pub struct TextInput {}

impl Input for TextInput {
    fn read(&mut self) -> Option<GameCommand> {
        let mut line = String::new();
        print!("Please enter direction: ");
        stdout().flush().unwrap();

        let stdin = std::io::stdin();
        let _ = stdin.read_line(&mut line).unwrap();
        //let _ = stdin.lock().read_line(&mut line).unwrap();
        line = line.trim().to_string();
        match line.as_str() {
            "l" => Some(GameCommand::Left),
            "r" => Some(GameCommand::Right),
            "s" => Some(GameCommand::Select),
            "q" => Some(GameCommand::Left),
            "f" => Some(GameCommand::Right),
            "w" => Some(GameCommand::Select),
            "quit" => Some(GameCommand::Quit),
            _ => {
                //println!("Invalid input");
                None
            }
        }
    }
}
/*stdout()
    .execute(SetForegroundColor(Color::Yellow))?
    .execute(SetBackgroundColor(Color::Blue))?
    .execute(Print("Styled text here."))?
    .execute(ResetColor)?;
let mut active_column = 0;*/

pub struct ColorOutput {}

impl Output for ColorOutput {
    fn write(&self, board: &RenderBoard) -> Result<()> {
        for row in (0..board.size()).rev() {
            for col in 0..board.size() {
                let v = board.get(col, row);
                let color = Color::Rgb {
                    r: v.r,
                    g: v.g,
                    b: v.b,
                };
                //if col == self.active_col {
                //    stdout().execute(SetBackgroundColor(Color::Red))?;
                //}

                stdout()
                    .execute(SetBackgroundColor(Color::Green))?
                    .execute(SetForegroundColor(color))?
                    .execute(Print("▒"))?
                    .execute(ResetColor)?;
            }
            stdout().execute(Print("\n"))?;
        }
        //board.print();
        Ok(())
    }
}

pub struct ConsoleOutput {}

impl Output for ConsoleOutput {
    fn write(&self, board: &RenderBoard) -> Result<()> {
        //board.print();
        for row in board.cells.iter() {
            for cell in row.iter() {
                print!("{:?} ", cell);
            }
            //println!();
        }
        //println!("");
        Ok(())
    }
}

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

    /*stdout()
        .execute(SetForegroundColor(Color::Yellow))?
        .execute(SetBackgroundColor(Color::Blue))?
        .execute(Print("Styled text here."))?
        .execute(ResetColor)?;
    let mut active_column = 0;*/

    ////println!("");
    //Ok(())
}

fn main() -> Result<()> {
    game_loop()?;
    let con_in = ConsoleInput {};
    let con_out = ConsoleOutput {};
    let mut game = ConnectFour::new(con_in, con_out);
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
