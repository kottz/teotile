use anyhow::Result;

use rand::Rng;

use std::time::Instant;

use crossterm::{
    cursor::MoveLeft,
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
    fn write(&self, render_board: &RenderBoard) -> Result<()>;
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
            "left" => Some(GameCommand::Left),
            "right" => Some(GameCommand::Right),
            "select" => Some(GameCommand::Select),
            "quit" => Some(GameCommand::Quit),
            _ => {
                println!("Invalid input");
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
            println!();
        }
        println!("");
        Ok(())
    }
}

// Update the render functions to make
// use of the states. Makes it more cleaner to move
// the game into the winning state and then match
// on that in the render function
enum ConnectFourState {
    Start,
    Playing,
    Win,
    Finished,
}

pub struct ConnectFour<I: Input, O: Output> {
    input: I,
    output: O,
    board: GameBoard,
    in_a_row: usize,
    active_player: Cell,
    active_col: usize,
    finished: bool,
}

impl<I: Input, O: Output> Game for ConnectFour<I, O> {
    fn process_input(&mut self) -> Result<()> {
        let command = self.input.read();
        match command {
            Some(GameCommand::Left) => {
                println!("Go left");
                let _ = self.move_col(GameCommand::Left);
            }
            Some(GameCommand::Right) => {
                println!("Go right");
                let _ = self.move_col(GameCommand::Right);
            }
            Some(GameCommand::Select) => {
                println!("Go right");
                match self.make_move(self.active_col, self.active_player) {
                    Ok(place) => {
                        println!("Placed at {:?}", place);
                        let win = self.check_win(place, self.in_a_row);
                        if win.is_some() {
                            println!("Player {:?} wins!", self.active_player);
                            self.finished = true;
                        }
                        self.active_player = match self.active_player {
                            Cell::PlayerX => Cell::PlayerO,
                            Cell::PlayerO => Cell::PlayerX,
                            _ => Cell::Empty,
                        };
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
            Some(GameCommand::Quit) => {
                self.finished = true;
                //println!("Quitting");
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

    fn render(&self) -> Result<()> {
        println!(
            "active col: {:?}, active player {:?}",
            self.active_col, self.active_player
        );
        // I want to create a renderboard from the gameboard here and then just send it to the
        // output function. It will handle the final rendering to either the terminal or the LEDs.

        let mut render_board = RenderBoard::new(RGB::new(0, 0, 0));

        for row in 0..self.board.size() {
            for col in 0..self.board.size() {
                let cell = self.board.get(col, row);
                let mut rgb = RGB::new(0, 0, 0);
                if col == self.active_col {
                    rgb = RGB::new(0, 255, 0);
                }
                match cell {
                    Cell::PlayerX => {
                        rgb = RGB::new(255, 0, 0);
                    }
                    Cell::PlayerO => {
                        rgb = RGB::new(0, 0, 255);
                    }
                    _ => {}
                }
                render_board.set(col, row, rgb);
            }
        }
        //Later I want to make the outer loop handle all of the timings
        //Refactor this when you add a realtime game.
        if self.finished {
            let mut rng = rand::thread_rng();
            for i in 0..30 {
                for row in 0..self.board.size() {
                    for col in 0..self.board.size() {
                        let rgb = RGB::new(
                            rng.gen_range(0..=255),
                            rng.gen_range(0..=255),
                            rng.gen_range(0..=255),
                        );
                        render_board.set(col, row, rgb);
                    }
                }
                let start = Instant::now();
                self.output.write(&render_board)?;
                let elapsed = start.elapsed();

                let frame_time = std::time::Duration::from_millis(1000 / 10);
                if elapsed < frame_time {
                    std::thread::sleep(frame_time - elapsed);
                }
                println!();
            }
        }

        self.output.write(&render_board)?;
        Ok(())
    }
}
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

impl<I: Input, O: Output> ConnectFour<I, O> {
    pub fn new(input: I, output: O) -> Self {
        Self {
            input: input,
            output: output,
            board: GameBoard::new(),
            in_a_row: 4,
            active_player: Cell::PlayerX,
            active_col: 0,
            finished: false,
        }
    }

    pub fn move_col(&mut self, direction: GameCommand) -> Result<()> {
        if direction == GameCommand::Left && self.active_col > 0 {
            self.active_col -= 1;
        }
        if direction == GameCommand::Right && self.active_col < self.board.size() - 1 {
            self.active_col += 1;
        }
        Ok(())
    }

    pub fn make_move(&mut self, x: usize, player: Cell) -> Result<(usize, usize)> {
        if x > self.board.size() {
            return Err(anyhow::anyhow!("Invalid move"));
        }
        //if player != self.active_player {
        //    return Err(anyhow::anyhow!("Not your turn"));
        //}
        if self.board.get(x, self.board.size() - 1) != Cell::Empty {
            return Err(anyhow::anyhow!("Column is full"));
        }
        let mut place: (usize, usize) = (x, 0);
        for y in (0..self.board.size()).rev() {
            if self.board.get(x, y) != Cell::Empty {
                self.board.set(x, y + 1, player);
                place = (x, y + 1);
                break;
            }
            // If we're at the last row, and the cell is empty, then place there
            if y == 0 {
                self.board.set(x, y, player);
                place = (x, y);
            }
            //println!("x: {}, y: {}", x, y);
        }
        Ok(place)
    }

    fn check_line(&self, x: i32, y: i32, dx: i32, dy: i32, player: Cell, in_a_row: usize) -> Option<Vec<(usize, usize)>> {
        let mut positions = Vec::new();

        for i in -(in_a_row as i32 - 1)..(in_a_row as i32) {
            let nx = x + i * dx;
            let ny = y + i * dy;

            if nx >= 0
                && ny >= 0
                && nx < self.board.size() as i32
                && ny < self.board.size() as i32
                && self.board.get(nx as usize, ny as usize) == player
            {
                positions.push((nx as usize, ny as usize));
                if positions.len() >= in_a_row {
                    return Some(positions);
                }
            } else {
                positions.clear();
            }
        }
        None
    }

    pub fn check_win(&self, last_move: (usize, usize), in_a_row: usize) -> Option<(Cell, Vec<(usize, usize)>)> {
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
            if let Some(winning_line) = self.check_line(x as i32, y as i32, dx, dy, player, in_a_row) {
                println!("Winning line: {:?}", winning_line);
                return Some((player, winning_line));
            }
        }
        None
    }
}

pub trait Game {
    fn process_input(&mut self) -> Result<()>;
    fn update(&mut self) -> Result<()>;
    fn render(&self) -> Result<()>;
}

pub struct Board<T> {
    cells: [[T; 12]; 12],
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
    cells: [[Cell; 12]; 12],
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
                    Cell::Empty => print!(". "),
                    Cell::PlayerX => print!("X "),
                    Cell::PlayerO => print!("O "),
                }
            }
            println!();
        }
        println!("");
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
                println!();
            }
            println!();
        }
    */
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
    let text_in = TextInput {};
    let con_out = ColorOutput {};
    let mut game = ConnectFour::new(text_in, con_out);

    loop {
        game.process_input()?;
        game.update()?;
        game.render()?;

        if game.finished {
            let text_in = TextInput {};
            let con_out = ColorOutput {};
            game = ConnectFour::new(text_in, con_out);
            //break;
        }
    }
    //Ok(())

    /*stdout()
        .execute(SetForegroundColor(Color::Yellow))?
        .execute(SetBackgroundColor(Color::Blue))?
        .execute(Print("Styled text here."))?
        .execute(ResetColor)?;
    let mut active_column = 0;*/

    //println!("");
    //Ok(())
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
