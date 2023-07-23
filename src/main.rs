use anyhow::Result;

use crossterm::{
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};
use std::io::{stdout, Write};

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

pub enum GameCommand {
    Left,
    Right,
    Down,
    Up,
    Quit,
    Select,
}

pub trait Input {
    fn read(&mut self) -> Option<GameCommand>;
}

pub trait Output {
    fn write(&self, state: &GameBoard) -> Result<()>;
}

pub struct ConsoleInput {
    //buffer: String,
}

impl Input for ConsoleInput {
    fn read(&mut self) -> Option<GameCommand> {
        let event = read().unwrap();

        println!("Event::{:?}\r", event);

        if event == Event::Key(KeyCode::Char('q').into()) {
            println!("quit");
            return Some(GameCommand::Quit);
        }
        if event == Event::Key(KeyCode::Char('w').into()) {
            println!("up");
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

pub struct ConsoleOutput {}

impl Output for ConsoleOutput {
    fn write(&self, state: &GameBoard) -> Result<()> {
        state.print();
        Ok(())
    }
}

pub struct ConnectFour<I: Input, O: Output> {
    input: I,
    output: O,
    board: GameBoard,
    active_player: Cell,
}

impl<I: Input, O: Output> Game for ConnectFour<I, O> {
    fn process_input(&mut self, input: &mut impl Input) -> Result<()> {
        let command = input.read();
        match command {
            Some(GameCommand::Quit) => {
                println!("Quitting");
                std::process::exit(0);
            }
            Some(GameCommand::Up) => {
                println!("Up");
            }
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        Ok(())
    }

    fn render(&self, output: &impl Output) -> Result<()> {
        output.write(&self.board)?;
        Ok(())
    }
}

impl<I: Input, O: Output> ConnectFour<I, O> {
    pub fn new(input: I, output: O) -> Self {
        Self {
            input: input,
            output: output,
            board: GameBoard::new(),
            active_player: Cell::PlayerX,
        }
    }

    pub fn make_move(&mut self, x: usize, player: Cell) -> Result<()> {
        if x > self.board.size() {
            return Err(anyhow::anyhow!("Invalid move"));
        }
        //if player != self.active_player {
        //    return Err(anyhow::anyhow!("Not your turn"));
        //}
        if self.board.get(x, self.board.size() - 1) != Cell::Empty {
            return Err(anyhow::anyhow!("Column is full"));
        }
        for y in (0..self.board.size()).rev() {
            if self.board.get(x, y) != Cell::Empty {
                self.board.set(x, y + 1, player);
                break;
            }
            // If we're at the last row, and the cell is empty, then place there
            if y == 0 {
                self.board.set(x, y, player);
            }
            //println!("x: {}, y: {}", x, y);
        }
        self.active_player = match self.active_player {
            Cell::PlayerX => Cell::PlayerO,
            Cell::PlayerO => Cell::PlayerX,
            Cell::Empty => Cell::Empty,
        };
        Ok(())
    }

    fn check_line(&self, x: i32, y: i32, dx: i32, dy: i32, player: Cell, in_a_row: usize) -> bool {
        let mut count = 0;

        for i in -(in_a_row as i32 - 1)..(in_a_row as i32) {
            let nx = x + i * dx;
            let ny = y + i * dy;

            if nx >= 0
                && ny >= 0
                && nx < self.board.size() as i32
                && ny < self.board.size() as i32
                && self.board.get(nx as usize, ny as usize) == player
            {
                count += 1;
                if count >= in_a_row {
                    return true;
                }
            } else {
                count = 0;
            }
        }
        false
    }

    pub fn check_win(&self, last_move: (usize, usize), in_a_row: usize) -> Option<Cell> {
        let (x, y) = last_move;
        let directions = vec![
            (0, 1),  // up
            (1, 0),  // right
            (1, 1),  // up-right
            (1, -1), // down-right
        ];
        let player = self.board.get(x, y);

        if player == Cell::Empty {
            return None;
        }

        for (dx, dy) in directions {
            if self.check_line(x as i32, y as i32, dx, dy, player, in_a_row) {
                return Some(player);
            }
        }
        None
    }
}

pub trait Game {
    fn process_input(&mut self, input: &mut impl Input) -> Result<()>;
    fn update(&mut self) -> Result<()>;
    fn render(&self, output: &impl Output) -> Result<()>;
}

pub struct GameBoard {
    cells: [[Cell; 12]; 12],
    active_player: Cell,
}

impl GameBoard {
    pub fn new() -> Self {
        Self {
            cells: Default::default(),
            active_player: Cell::PlayerX,
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
                    Cell::Empty => print!(". "),
                    Cell::PlayerX => print!("X "),
                    Cell::PlayerO => print!("O "),
                }
            }
            println!();
        }
        println!("");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let con_in = ConsoleInput {};
        let con_out = ConsoleOutput {};
        let game = ConnectFour::new(con_in, con_out);

        // The board should be empty at the start of the game
        for row in game.board.cells.iter() {
            for cell in row.iter() {
                assert_eq!(*cell, Cell::Empty);
            }
        }

        // The active player should be PlayerX at the start of the game
        assert_eq!(game.active_player, Cell::PlayerX);
    }
    /*
        #[test]
        fn test_make_move_turn() {
            let con_in = ConsoleInput {};
            let con_out = ConsoleOutput {};
            let mut game = ConnectFour::new(con_in, con_out);

            // PlayerX should be able to make a move
            assert!(game.make_move(0, Cell::PlayerX).is_ok());

            // Now the active player should be PlayerO
            assert_eq!(game.active_player, Cell::PlayerO);

            // PlayerO should be able to make a move
            assert!(game.make_move(1, Cell::PlayerO).is_ok());

            // Now the active player should be PlayerX again
            assert_eq!(game.active_player, Cell::PlayerX);

            // PlayerO should not be able to make a move because it's not their turn
            assert!(game.make_move(2, Cell::PlayerO).is_err());
        }
    */
    #[test]
    fn test_make_move_column_full() {
        let con_in = ConsoleInput {};
        let con_out = ConsoleOutput {};
        let mut game = ConnectFour::new(con_in, con_out);

        // Fill up the first column
        for _ in 0..game.board.size() {
            game.make_move(0, Cell::PlayerX).unwrap();
            game.make_move(1, Cell::PlayerO).unwrap();
        }

        // The first column should be full
        assert!(game.make_move(0, Cell::PlayerX).is_err());
    }

    #[test]
    fn test_check_win_horizontal() {
        let con_in = ConsoleInput {};
        let con_out = ConsoleOutput {};
        let mut game = ConnectFour::new(con_in, con_out);

        // No one should have won the game yet
        assert_eq!(game.check_win((0, 0), 5), None);

        // PlayerX makes five moves in a row
        for i in 0..5 {
            game.make_move(i, Cell::PlayerX).unwrap();
            game.make_move(i, Cell::PlayerO).unwrap();
        }
        game.board.set_cell(game.board.size() - 1, 0, Cell::PlayerO);
        // PlayerX should have won the game
        assert_eq!(game.check_win((0, 1), 5), Some(Cell::PlayerO));
    }

    #[test]
    fn test_check_win_vertical() {
        let con_in = ConsoleInput {};
        let con_out = ConsoleOutput {};
        let mut game = ConnectFour::new(con_in, con_out);

        assert_eq!(game.check_win((4, 2), 5), None);

        for _ in 0..5 {
            game.make_move(4, Cell::PlayerX).unwrap();
            game.make_move(6, Cell::PlayerO).unwrap();
        }
        assert_eq!(game.check_win((4, 2), 5), Some(Cell::PlayerX));
    }

    #[test]
    fn test_check_win_diagonal() {
        let con_in = ConsoleInput {};
        let con_out = ConsoleOutput {};
        let mut game = ConnectFour::new(con_in, con_out);

        for i in 0..5 {
            assert_eq!(game.check_win((i, i), 5), None);
        }
        for i in 0..5 {
            assert_eq!(game.check_win((6 + i, 4 - i), 5), None);
        }

        for x in 1..5 {
            for y in 0..x {
                game.make_move(x, Cell::PlayerX).unwrap();
                game.make_move(6 + y, Cell::PlayerO).unwrap();
            }
        }
        for x in 0..5 {
            game.make_move(6 + x, Cell::PlayerX).unwrap();
            game.make_move(x, Cell::PlayerO).unwrap();
        }

        for i in 0..5 {
            assert_eq!(game.check_win((i, i), 5), Some(Cell::PlayerO));
        }
        for i in 0..5 {
            assert_eq!(game.check_win((6 + i, 4 - i), 5), Some(Cell::PlayerX));
        }
    }
}

fn game_loop() -> Result<()> {
    let mut game = GameBoard::new();

    // using the macro
    /*execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        SetBackgroundColor(Color::Red),
        Print("Styled text here."),
        ResetColor
    )?;*/

    // or using functions
    stdout()
        .execute(SetForegroundColor(Color::Yellow))?
        .execute(SetBackgroundColor(Color::Blue))?
        .execute(Print("Styled text here."))?
        .execute(ResetColor)?;
    let mut active_column = 0;

    println!("");
    Ok(())
}

fn main() -> Result<()> {
    game_loop()?;
    let con_in = ConsoleInput {};
    let con_out = ConsoleOutput {};
    let mut game = ConnectFour::new(con_in, con_out);
    let win = game.check_win((0, 0), 4);
    println!("win: {:?}", win);
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
    println!("win: {:?}", win);
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
    println!("win: {:?}", win);

    Ok(())
}
