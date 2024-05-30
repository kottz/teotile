use crate::game::{Board, ButtonState, Cell, CommandType, Game, GameCommand};
use crate::RenderBoard;
use crate::GRID_SIZE;
use crate::RGB;
use anyhow::Result;
use core::time::Duration;
use libm::{fabs, sin};
use smallvec::SmallVec;

use super::Player;

const WIN_ANIMATION_SPEED: Duration = Duration::from_millis(50);

#[derive(Debug, PartialEq)]
pub enum ConnectFourState {
    Playing,
    Win(SmallVec<[(usize, usize); GRID_SIZE]>),
    Finished,
}

#[derive(Debug, PartialEq)]
struct WinAnimationState {
    state: usize,
    last_update_time: Duration,
}

pub type GameBoard = Board<Cell, GRID_SIZE, GRID_SIZE>;

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
                if input_command.player != self.active_player {
                    return Ok(());
                }

                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Left => {
                            let _ = self.move_col(CommandType::Left);
                        }
                        CommandType::Right => {
                            let _ = self.move_col(CommandType::Right);
                        }
                        CommandType::Select => {
                            if let Ok(place) = self.make_move(self.active_col, self.active_player) {
                                let win = self.check_win(place, self.in_a_row);
                                if let Some((_, winning_line)) = win {
                                    self.state = ConnectFourState::Win(winning_line);
                                }
                                self.active_player = match self.active_player {
                                    Player::Player1 => Player::Player2,
                                    Player::Player2 => Player::Player1,
                                };
                            }
                        }
                        CommandType::Quit => {
                            self.state = ConnectFourState::Finished;
                        }
                        _ => {}
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
            }
            ConnectFourState::Finished => {}
        }

        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        self.current_time += delta_time;

        match &self.state {
            ConnectFourState::Playing => {
                self.win_animation_state.last_update_time = self.current_time;
            }
            ConnectFourState::Win(_) => {
                if self.current_time - self.win_animation_state.last_update_time
                    > WIN_ANIMATION_SPEED
                {
                    self.win_animation_state.last_update_time = self.current_time;

                    if self.win_animation_state.state >= 20 {
                        self.win_animation_state.state = 0;
                    }
                    self.win_animation_state.state += 1;
                }
            }
            ConnectFourState::Finished => {}
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
        //let mut render_board = RenderBoard::new(RGB::new(0, 0, 0));
        let mut render_board = RenderBoard::new();

        match &self.state {
            ConnectFourState::Playing => {
                for row in 0..self.board.rows() {
                    for col in 0..self.board.cols() {
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
                // reset color in active column
                for row in 0..self.board.rows() {
                    for col in 0..self.board.cols() {
                        if col == self.active_col && self.board.get(col, row) == Cell::Empty {
                            let rgb = RGB::new(0, 0, 0);
                            render_board.set(col, row, rgb);
                        }
                    }
                }

                let s = self.win_animation_state.state;
                let f: f64 = s as f64;
                let s = fabs(sin(f * 2.0 * 3.141 / 20.0)) * 10.0 + 10.0;
                let color = RGB::new(s as u8 * 10, s as u8 * 10, s as u8 * 10);
                for (col, row) in winning_line {
                    render_board.set(*col, *row, color);
                }
            }
            ConnectFourState::Finished => {}
        }
        Ok(render_board)
    }
}

impl ConnectFour {
    pub fn new() -> Self {
        Self {
            board: GameBoard::new(),
            in_a_row: 4,
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
        if direction == CommandType::Right && self.active_col < self.board.cols() - 1 {
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
        if x > self.board.cols() {
            return Err(anyhow::anyhow!("Invalid move"));
        }

        if self.board.get(x, self.board.cols() - 1) != Cell::Empty {
            return Err(anyhow::anyhow!("Column is full"));
        }
        let mut place: (usize, usize) = (x, 0);
        for y in (0..self.board.rows()).rev() {
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
                && nx < self.board.cols() as i32
                && ny < self.board.rows() as i32
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

impl Default for ConnectFour {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = ConnectFour::new();

        // The board should be empty at the start of the game
        for row in game.board.cells.iter() {
            for cell in row.iter() {
                assert_eq!(*cell, Cell::Empty);
            }
        }

        // The active player should be PlayerX at the start of the game
        assert_eq!(game.active_player, Player::Player1);
    }

    #[test]
    fn test_make_move_column_full() {
        let mut game = ConnectFour::new();

        // Fill up the first column
        for _ in 0..game.board.cols() {
            game.make_move(0, Player::Player1).unwrap();
            game.make_move(1, Player::Player2).unwrap();
        }

        // The first column should be full
        assert!(game.make_move(0, Player::Player1).is_err());
    }

    #[test]
    fn test_check_win_horizontal() {
        let mut game = ConnectFour::new();

        // No one should have won the game yet
        assert_eq!(game.check_win((0, 0), 5), None);

        // PlayerX makes five moves in a row
        for i in 0..5 {
            game.make_move(i, Player::Player1).unwrap();
            game.make_move(i, Player::Player2).unwrap();
        }
        game.board.set(game.board.cols() - 1, 0, Cell::PlayerO);
        // PlayerX should have won the game
        //(Option<(Cell, Vec<(usize, usize)>)> {
        let (win_cell, _) = game.check_win((0, 1), 5).unwrap();
        assert_eq!(win_cell, (Cell::PlayerO));
    }

    #[test]
    fn test_check_win_vertical() {
        let mut game = ConnectFour::new();

        assert_eq!(game.check_win((4, 2), 5), None);

        for _ in 0..5 {
            game.make_move(4, Player::Player1).unwrap();
            game.make_move(6, Player::Player2).unwrap();
        }
        let (win_cell, _) = game.check_win((4, 2), 5).unwrap();
        assert_eq!(win_cell, (Cell::PlayerX));
    }

    #[test]
    fn test_check_win_diagonal() {
        let mut game = ConnectFour::new();

        for i in 0..5 {
            assert_eq!(game.check_win((i, i), 5), None);
        }
        for i in 0..5 {
            assert_eq!(game.check_win((6 + i, 4 - i), 5), None);
        }

        for x in 1..5 {
            for y in 0..x {
                game.make_move(x, Player::Player1).unwrap();
                game.make_move(6 + y, Player::Player2).unwrap();
            }
        }
        for x in 0..5 {
            game.make_move(6 + x, Player::Player1).unwrap();
            game.make_move(x, Player::Player2).unwrap();
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
