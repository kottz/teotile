use crate::game::{ButtonState, Cell, CommandType, Game, GameBoard, GameCommand};
//use crate::io::Output;
//use crate::Input;
use crate::RenderBoard;
use crate::GRID_SIZE;
use crate::RGB;
use anyhow::Result;
use core::time::Duration;
use rand::Rng;
use smallvec::SmallVec;

use super::Player;

#[derive(Debug, PartialEq)]
pub enum ConnectFourState {
    //Start,
    Playing,
    Win(SmallVec<[(usize, usize); GRID_SIZE]>),
    Finished,
}

#[derive(Debug, PartialEq)]
struct WinAnimationState {
    state: usize,
    last_update_time: Duration,
}

pub struct ConnectFour {
    pub board: GameBoard,
    in_a_row: usize,
    active_player: Player,
    active_col: usize,
    pub state: ConnectFourState,
    win_animation_state: WinAnimationState,
    current_time: Duration,
}

impl Game for ConnectFour {
    fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
        match &self.state {
            ConnectFourState::Playing => {
                // Check if the input is from the active player
                if input_command.player != self.active_player {
                    // Ignore inputs from the inactive player
                    return Ok(());
                }

                // Process the input command only if it's a button press
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Left => {
                            let _ = self.move_col(CommandType::Left);
                        }
                        CommandType::Right => {
                            let _ = self.move_col(CommandType::Right);
                        }
                        CommandType::Select => {
                            match self.make_move(self.active_col, self.active_player) {
                                Ok(place) => {
                                    let win = self.check_win(place, self.in_a_row);
                                    if let Some((_, winning_line)) = win {
                                        self.state = ConnectFourState::Win(winning_line);
                                    }
                                    self.active_player = match self.active_player {
                                        Player::Player1 => Player::Player2,
                                        Player::Player2 => Player::Player1,
                                    };
                                }
                                Err(e) => {
                                    // Handle the error if needed
                                    // println!("Error: {:?}", e);
                                }
                            }
                        }
                        CommandType::Quit => {
                            self.state = ConnectFourState::Finished;
                        }
                        _ => {
                            // Ignore other commands
                        }
                    }
                }
            }
            ConnectFourState::Win(_) => {
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Select => {
                            self.state = ConnectFourState::Playing;
                            self.board = GameBoard::new();
                            self.active_player = Player::Player1;
                            self.active_col = 0;
                        }
                        _ => return Ok(()),
                    }
                }
                //self.process_input_win(input_command)?;
            }
            ConnectFourState::Finished => {
                //self.process_input_finished(input_command)?;
            }
        }

        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        self.current_time += delta_time;

        // keep track of win animation timing here maybe?
        match &self.state {
            ConnectFourState::Playing => {
                self.win_animation_state.last_update_time = self.current_time;

                //todo
            }
            ConnectFourState::Win(_) => {
                if self.current_time - self.win_animation_state.last_update_time > Duration::from_millis(100) {
                    self.win_animation_state.last_update_time = self.current_time;

                    if self.win_animation_state.state >= 20 {
                        self.win_animation_state.state = 0;
                    }
                    self.win_animation_state.state += 1;
                } //todo
            }
            ConnectFourState::Finished => {
                //todo
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
        //println!(
        //    "active col: {:?}, active player {:?}",
        //    self.active_col, self.active_player
        //);
        // I want to create a renderboard from the gameboard here and then just send it to the
        // output function. It will handle the final rendering to either the terminal or the LEDs.

        let mut render_board = RenderBoard::new(RGB::new(0, 0, 0));

        //Later I want to make the outer loop handle all of the timings
        //Refactor this when you add a realtime game.
        match &self.state {
            // ConnectFourState::Start => {
            //     //todo
            // }
            ConnectFourState::Playing => {
                for row in 0..self.board.size() {
                    for col in 0..self.board.size() {
                        let cell = self.board.get(col, row);
                        let mut rgb = RGB::new(0, 0, 0);
                        if col == self.active_col {
                            match &self.active_player {
                                Player::Player1 => {
                                    rgb = RGB::new(230, 166, 83);
                                }
                                Player::Player2 => {
                                    rgb = RGB::new(125, 113, 191);
                                }
                            }
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
            }
            ConnectFourState::Win(winning_line) => {
                // reset color in active column!
                for row in 0..self.board.size() {
                    for col in 0..self.board.size() {
                        if col == self.active_col && self.board.get(col, row) == Cell::Empty {
                            let rgb = RGB::new(0, 0, 0);
                            render_board.set(col, row, rgb);
                        }
                    }
                }
                let mut rng = rand::thread_rng();
                for _ in 0..20 {
                    /*
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
                    */
                    let rgb = RGB::new(
                        rng.gen_range(0..=255),
                        rng.gen_range(0..=255),
                        rng.gen_range(0..=255),
                    );
                    for (col, row) in winning_line {
                        render_board.set(*col, *row, rgb);
                    }
                    //render the winning line separately
                    /*
                    for (col, row) in winning_line {
                        let rgb = RGB::new(0, 0, 0);
                        render_board.set(*col, *row, rgb);
                    }*/

                    //let start = Instant::now();
                    //self.output.write(&render_board)?;
                    //let elapsed = start.elapsed();

                    //let frame_time = std::time::Duration::from_millis(1000 / 10);
                    //if elapsed < frame_time {
                    //    std::thread::sleep(frame_time - elapsed);
                    //}
                    //println!();
                }
            }
            ConnectFourState::Finished => {
                //todo
            }
        }

        //self.output.write(&render_board)?;
        Ok(render_board)
    }
}

impl ConnectFour {
    pub fn new() -> Self {
        Self {
            board: GameBoard::new(),
            in_a_row: 4,
            //active_player: Cell::PlayerX,
            active_player: Player::Player1,
            active_col: 0,
            state: ConnectFourState::Playing,
            win_animation_state: WinAnimationState {
                state: 0,
                last_update_time: Duration::from_millis(0),
            },
            current_time: Duration::from_millis(0),
        }
    }

    pub fn move_col(&mut self, direction: CommandType) -> Result<()> {
        if direction == CommandType::Left && self.active_col > 0 {
            self.active_col -= 1;
        }
        if direction == CommandType::Right && self.active_col < self.board.size() - 1 {
            self.active_col += 1;
        }
        Ok(())
    }

    pub fn _get_cell_from_player(&self, player: Player) -> Cell {
        match player {
            Player::Player1 => Cell::PlayerX,
            Player::Player2 => Cell::PlayerO,
        }
    }

    pub fn make_move(&mut self, x: usize, player: Player) -> Result<(usize, usize)> {
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
                self.board.set(x, y + 1, self._get_cell_from_player(player));
                place = (x, y + 1);
                break;
            }
            // If we're at the last row, and the cell is empty, then place there
            if y == 0 {
                self.board.set(x, y, self._get_cell_from_player(player));
                place = (x, y);
            }
            ////println!("x: {}, y: {}", x, y);
        }
        Ok(place)
    }

    fn check_line(
        &self,
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
        player: Cell,
        in_a_row: usize,
    ) -> Option<SmallVec<[(usize, usize); GRID_SIZE]>> {
        let mut positions = SmallVec::<[(usize, usize); GRID_SIZE]>::new();

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

    pub fn check_win(
        &self,
        last_move: (usize, usize),
        in_a_row: usize,
    ) -> Option<(Cell, SmallVec<[(usize, usize); GRID_SIZE]>)> {
        let (x, y) = last_move;
        let directions = [
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
            if let Some(winning_line) =
                self.check_line(x as i32, y as i32, dx, dy, player, in_a_row)
            {
                ////println!("Winning line: {:?}", winning_line);
                return Some((player, winning_line));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConsoleInput, ConsoleOutput};

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
        //(Option<(Cell, Vec<(usize, usize)>)> {
        let (win_cell, _) = game.check_win((0, 1), 5).unwrap();
        assert_eq!(win_cell, (Cell::PlayerO));
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
        let (win_cell, _) = game.check_win((4, 2), 5).unwrap();
        assert_eq!(win_cell, (Cell::PlayerX));
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
            let (win_cell, _) = game.check_win((i, i), 5).unwrap();
            assert_eq!(win_cell, (Cell::PlayerO));
        }
        for i in 0..5 {
            let (win_cell, _) = game.check_win((6 + i, 4 - i), 5).unwrap();
            assert_eq!(win_cell, (Cell::PlayerX));
        }
    }
}
