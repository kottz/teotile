#[cfg(feature = "std")]
use std::io::{stdout, Write};

#[cfg(feature = "std")]
use crossterm::{
    cursor::MoveLeft,
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};

use crate::GameCommand;
use crate::RenderBoard;

use anyhow::Result;

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
