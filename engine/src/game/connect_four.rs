use crate::animation::Animation;
use crate::game::{Board, ButtonState, CommandType, Game, GameCommand};
use crate::GRID_SIZE;
use crate::RGB;
use crate::{GameError, RenderBoard};
use core::time::Duration;
use smallvec::SmallVec;

const WIN_ANIMATION_SPEED: Duration = Duration::from_millis(50);

use super::Player;

#[derive(Debug, PartialEq)]
pub enum ConnectFourState {
    Playing,
    Win(SmallVec<[(usize, usize); GRID_SIZE]>),
    Tie,
    Finished,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    #[default]
    Empty,
    PlayerX,
    PlayerO,
}

pub type GameBoard = Board<Cell, 7, 6>;

pub struct ConnectFour {
    pub board: GameBoard,
    in_a_row: usize,
    active_player: Player,
    active_col: usize,
    pub state: ConnectFourState,
    win_animation: Animation,
    current_time: Duration,
}

impl ConnectFour {
    pub fn new() -> Self {
        Self {
            board: GameBoard::new(),
            in_a_row: 4,
            active_player: Player::Player1,
            active_col: 0,
            state: ConnectFourState::Playing,
            win_animation: Animation::new(WIN_ANIMATION_SPEED),
            current_time: Duration::from_millis(0),
        }
    }

    pub fn move_col(&mut self, direction: CommandType) -> Result<(), GameError> {
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

    pub fn make_move(&mut self, x: usize, player: Player) -> Result<(usize, usize), GameError> {
        if x > self.board.cols() {
            return Err(GameError::OutOfBounds);
        }
        // println!(
        //     "Making move x: {}, rows() - 1: {}",
        //     x,
        //     self.board.rows() - 1
        // );
        if self.board.get(x, self.board.rows() - 1) != Cell::Empty {
            return Err(GameError::InvalidMove);
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
                return Some((player, winning_line));
            }
        }
        None
    }

    fn check_draw(&self) -> bool {
        for col in 0..self.board.cols() {
            if self.board.get(col, self.board.rows() - 1) == Cell::Empty {
                return false;
            }
        }
        true
    }
}

impl Default for ConnectFour {
    fn default() -> Self {
        Self::new()
    }
}
impl Game for ConnectFour {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
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
                                } else if self.check_draw() {
                                    self.state = ConnectFourState::Tie;
                                } else {
                                    self.active_player = match self.active_player {
                                        Player::Player1 => Player::Player2,
                                        Player::Player2 => Player::Player1,
                                    };
                                }
                            }
                        }
                        CommandType::Quit => {
                            self.state = ConnectFourState::Finished;
                        }
                        _ => {}
                    }
                }
            }
            ConnectFourState::Win(_) | ConnectFourState::Tie => {
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

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.current_time += delta_time;

        match &self.state {
            ConnectFourState::Playing => {}
            ConnectFourState::Win(_) | ConnectFourState::Tie => {
                self.win_animation.update(self.current_time);
            }
            ConnectFourState::Finished => {}
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        match &self.state {
            ConnectFourState::Playing => {
                for col in 0..self.board.cols() {
                    for row in 0..self.board.rows() {
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
                for col in 0..self.board.cols() {
                    for row in 0..self.board.rows() {
                        if col == self.active_col && self.board.get(col, row) == Cell::Empty {
                            let rgb = RGB::new(0, 0, 0);
                            render_board.set(col, row, rgb);
                        }
                    }
                }

                let color = self.win_animation.get_color();
                for (col, row) in winning_line {
                    render_board.set(*col, *row, color);
                }
            }
            ConnectFourState::Tie => {
                for col in 0..self.board.cols() {
                    for row in 0..self.board.rows() {
                        let color = self.win_animation.get_color();
                        render_board.set(col, row, color);
                    }
                }
            }
            ConnectFourState::Finished => {}
        }
        Ok(render_board)
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
        for _ in 0..game.board.rows() {
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

        // No one should have won the game yet
        assert_eq!(game.check_win((0, 0), 4), None);

        // PlayerX makes four moves diagonally
        game.make_move(0, Player::Player1).unwrap(); // (0, 0)
        game.make_move(1, Player::Player2).unwrap();
        game.make_move(1, Player::Player1).unwrap(); // (1, 1)
        game.make_move(2, Player::Player2).unwrap();
        game.make_move(2, Player::Player2).unwrap();
        game.make_move(2, Player::Player1).unwrap(); // (2, 2)
        game.make_move(3, Player::Player2).unwrap();
        game.make_move(3, Player::Player2).unwrap();
        game.make_move(3, Player::Player2).unwrap();
        game.make_move(3, Player::Player1).unwrap(); // (3, 3)

        // PlayerX should have won the game with a diagonal
        let (win_cell, _) = game.check_win((3, 3), 4).unwrap();
        assert_eq!(win_cell, Cell::PlayerX);
    }

    #[test]
    fn test_check_win_diagonal_opposite() {
        let mut game = ConnectFour::new();

        // No one should have won the game yet
        assert_eq!(game.check_win((0, 0), 4), None);

        // PlayerX makes four moves diagonally (opposite direction)
        game.make_move(3, Player::Player1).unwrap(); // (3, 0)
        game.make_move(2, Player::Player2).unwrap();
        game.make_move(2, Player::Player1).unwrap(); // (2, 1)
        game.make_move(1, Player::Player2).unwrap();
        game.make_move(1, Player::Player2).unwrap();
        game.make_move(1, Player::Player1).unwrap(); // (1, 2)
        game.make_move(0, Player::Player2).unwrap();
        game.make_move(0, Player::Player2).unwrap();
        game.make_move(0, Player::Player2).unwrap();
        game.make_move(0, Player::Player1).unwrap(); // (0, 3)

        // PlayerX should have won the game with a diagonal in the opposite direction
        let (win_cell, _) = game.check_win((0, 3), 4).unwrap();
        assert_eq!(win_cell, Cell::PlayerX);
    }

    #[test]
    fn test_check_tie() {
        let mut game = ConnectFour::new();

        // This is not actually a draw but we are testing the check_draw function
        // with a full board.
        for col in 0..game.board.cols() {
            for row in 0..game.board.rows() {
                if row % 2 == 0 {
                    //game.board.set(col, row, Cell::PlayerX);
                    game.make_move(col, Player::Player1).unwrap();
                } else {
                    //game.board.set(col, row, Cell::PlayerO);
                    game.make_move(col, Player::Player2).unwrap();
                }
            }
        }

        // The game should be in a Tie state
        assert!(game.check_draw());
    }
}
