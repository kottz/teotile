use crate::animation::Animation;
use crate::game::{Board, ButtonState, CommandType, Game, GameCommand};
use crate::RGB;
use crate::{GameError, RenderBoard};
use core::time::Duration;
use smallvec::SmallVec;

use super::Player;

const WIN_ANIMATION_SPEED: Duration = Duration::from_millis(50);

#[derive(Debug, PartialEq)]
pub enum TicTacToeState {
    Playing,
    Win(SmallVec<[(usize, usize); 3]>),
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

type TicTacToeBoard = Board<Cell, 3, 3>;

impl TicTacToeBoard {
    fn full(&self) -> bool {
        self.cells.iter().flatten().all(|&cell| cell != Cell::Empty)
    }
}

pub struct TicTacToe {
    pub board: TicTacToeBoard,
    active_player: Player,
    active_cell: (usize, usize),
    pub state: TicTacToeState,
    win_animation: Animation,
    current_time: Duration,
}

impl Game for TicTacToe {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        match &self.state {
            TicTacToeState::Playing => {
                if input_command.player != self.active_player {
                    return Ok(());
                }

                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Left => {
                            if self.active_cell.0 > 0 {
                                self.active_cell.0 -= 1;
                            }
                        }
                        CommandType::Right => {
                            if self.active_cell.0 < 2 {
                                self.active_cell.0 += 1;
                            }
                        }
                        CommandType::Up => {
                            if self.active_cell.1 < 2 {
                                self.active_cell.1 += 1;
                            }
                        }
                        CommandType::Down => {
                            if self.active_cell.1 > 0 {
                                self.active_cell.1 -= 1;
                            }
                        }
                        CommandType::Select => {
                            if self.board.get(self.active_cell.0, self.active_cell.1) == Cell::Empty
                            {
                                self.board.set(
                                    self.active_cell.0,
                                    self.active_cell.1,
                                    self._get_cell_from_player(self.active_player),
                                );
                                if let Some(winning_line) = self.check_win(self.active_cell) {
                                    self.state = TicTacToeState::Win(winning_line);
                                } else {
                                    self.active_player = match self.active_player {
                                        Player::Player1 => Player::Player2,
                                        Player::Player2 => Player::Player1,
                                    };
                                }
                            }
                            if self.check_tie(self.active_cell) {
                                self.state = TicTacToeState::Tie;
                            }
                        }
                        CommandType::Quit => {
                            self.state = TicTacToeState::Finished;
                        }
                    }
                }
            }
            TicTacToeState::Win(_) | TicTacToeState::Tie => {
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Select => {
                            self.state = TicTacToeState::Playing;
                            self.board = TicTacToeBoard::new(); // Reset for Tic Tac Toe
                            self.active_player = Player::Player1;
                            self.active_cell = (0, 0);
                        }
                        _ => return Ok(()),
                    }
                }
            }
            TicTacToeState::Finished => {}
        }

        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.current_time += delta_time;

        match &self.state {
            TicTacToeState::Playing => {}
            TicTacToeState::Win(_) | TicTacToeState::Tie => {
                self.win_animation.update(self.current_time);
            }
            TicTacToeState::Finished => {}
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        // Closure to draw 'X' centered at (cx, cy)
        let draw_x = |cx: usize, cy: usize, grid: &mut RenderBoard, color: RGB| {
            if cx > 0 && cy > 0 && cx < grid.cols() - 1 && cy < grid.rows() - 1 {
                grid.set(cy - 1, cx - 1, color);
                grid.set(cy - 1, cx + 1, color);
                grid.set(cy, cx, color);
                grid.set(cy + 1, cx - 1, color);
                grid.set(cy + 1, cx + 1, color);
            }
        };

        // Closure to draw 'O' centered at (cx, cy)
        let draw_o = |cx: usize, cy: usize, grid: &mut RenderBoard, color: RGB| {
            if cx > 0 && cy > 0 && cx < grid.cols() - 1 && cy < grid.rows() - 1 {
                grid.set(cy - 1, cx - 1, color);
                grid.set(cy - 1, cx, color);
                grid.set(cy - 1, cx + 1, color);
                grid.set(cy, cx - 1, color);
                grid.set(cy, cx + 1, color);
                grid.set(cy + 1, cx - 1, color);
                grid.set(cy + 1, cx, color);
                grid.set(cy + 1, cx + 1, color);
            }
        };

        match &self.state {
            TicTacToeState::Playing => {
                for row in 0..self.board.rows() {
                    for col in 0..self.board.cols() {
                        let cell = self.board.get(col, row);
                        match cell {
                            Cell::PlayerX => {
                                draw_x(
                                    row * 3 + 1,
                                    col * 3 + 1,
                                    &mut render_board,
                                    RGB::new(255, 0, 0),
                                );
                            }
                            Cell::PlayerO => {
                                draw_o(
                                    row * 3 + 1,
                                    col * 3 + 1,
                                    &mut render_board,
                                    RGB::new(0, 0, 255),
                                );
                            }
                            _ => {}
                        }

                        let mut rgb;
                        // draw placed pieces
                        match cell {
                            Cell::PlayerX => {
                                rgb = RGB::new(185, 101, 207);
                                draw_x(row * 3 + 1, col * 3 + 1, &mut render_board, rgb);
                            }
                            Cell::PlayerO => {
                                rgb = RGB::new(102, 204, 187);
                                draw_o(row * 3 + 1, col * 3 + 1, &mut render_board, rgb);
                            }
                            _ => {}
                        }

                        // draw active cell
                        if (col, row) == self.active_cell {
                            if self.board.get(col, row) != Cell::Empty {
                                rgb = RGB::new(255, 0, 0);
                            } else {
                                rgb = RGB::new(0, 255, 0);
                            }
                            match self.active_player {
                                Player::Player1 => {
                                    draw_x(row * 3 + 1, col * 3 + 1, &mut render_board, rgb);
                                }
                                Player::Player2 => {
                                    draw_o(row * 3 + 1, col * 3 + 1, &mut render_board, rgb);
                                }
                            }
                        }
                    }
                }
            }
            TicTacToeState::Win(winning_line) => {
                let color = self.win_animation.get_color();
                for (col, row) in winning_line {
                    match self.active_player {
                        Player::Player1 => {
                            draw_x(row * 3 + 1, col * 3 + 1, &mut render_board, color);
                        }
                        Player::Player2 => {
                            draw_o(row * 3 + 1, col * 3 + 1, &mut render_board, color);
                        }
                    }
                }
            }
            TicTacToeState::Tie => {
                let color = self.win_animation.get_color();
                for col in 0..self.board.cols() {
                    for row in 0..self.board.rows() {
                        match self.board.get(col, row) {
                            Cell::PlayerX => {
                                draw_x(row * 3 + 1, col * 3 + 1, &mut render_board, color);
                            }
                            Cell::PlayerO => {
                                draw_o(row * 3 + 1, col * 3 + 1, &mut render_board, color);
                            }
                            _ => {}
                        }
                    }
                }
            }
            TicTacToeState::Finished => {}
        }
        Ok(render_board)
    }
}

impl TicTacToe {
    pub fn new() -> Self {
        Self {
            board: TicTacToeBoard::new(),
            active_player: Player::Player1,
            active_cell: (0, 0),
            state: TicTacToeState::Playing,
            win_animation: Animation::new(WIN_ANIMATION_SPEED),
            current_time: Duration::from_millis(0),
        }
    }

    pub fn _get_cell_from_player(&self, player: Player) -> Cell {
        match player {
            Player::Player1 => Cell::PlayerX,
            Player::Player2 => Cell::PlayerO,
        }
    }

    fn _make_move(&mut self, cell: (usize, usize), player: Player) -> Result<(), GameError> {
        if self.board.get(cell.0, cell.1) != Cell::Empty {
            return Err(GameError::InvalidMove);
        }

        self.board
            .set(cell.0, cell.1, self._get_cell_from_player(player));
        Ok(())
    }

    pub fn check_win(&self, last_move: (usize, usize)) -> Option<SmallVec<[(usize, usize); 3]>> {
        let (x, y) = last_move;
        let player = self.board.get(x, y);
        if player == Cell::Empty {
            return None;
        }

        let directions = [
            // Rows
            [(0, 0), (0, 1), (0, 2)],
            [(1, 0), (1, 1), (1, 2)],
            [(2, 0), (2, 1), (2, 2)],
            // Columns
            [(0, 0), (1, 0), (2, 0)],
            [(0, 1), (1, 1), (2, 1)],
            [(0, 2), (1, 2), (2, 2)],
            // Diagonal (top-left to bottom-right)
            [(0, 0), (1, 1), (2, 2)],
            // Anti-diagonal (top-right to bottom-left)
            [(0, 2), (1, 1), (2, 0)],
        ];

        for direction in directions.iter() {
            let mut winning_cells = SmallVec::new();
            for &(dx, dy) in direction {
                if self.board.get(dx, dy) != player {
                    break;
                }
                winning_cells.push((dx, dy));
            }
            if winning_cells.len() == 3 {
                return Some(winning_cells);
            }
        }

        None
    }

    pub fn check_tie(&self, last_move: (usize, usize)) -> bool {
        if self.check_win(last_move).is_none() {
            return self.board.full();
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    #[test]
    fn test_new_game() {
        let game = TicTacToe::new();

        for row in game.board.cells.iter() {
            for cell in row.iter() {
                assert_eq!(*cell, Cell::Empty);
            }
        }

        assert_eq!(game.active_player, Player::Player1);
    }

    #[test]
    fn test_make_move() {
        let mut game = TicTacToe::new();

        assert!(game._make_move((0, 0), Player::Player1).is_ok());
        assert_eq!(game.board.get(0, 0), Cell::PlayerX);

        assert!(game._make_move((0, 0), Player::Player2).is_err());
    }

    #[test]
    fn test_check_win_horizontal() {
        let mut game = TicTacToe::new();

        game._make_move((0, 0), Player::Player1).unwrap();
        game._make_move((1, 0), Player::Player1).unwrap();
        game._make_move((2, 0), Player::Player1).unwrap();

        assert_eq!(
            game.check_win((0, 0)),
            Some(smallvec![(0, 0), (1, 0), (2, 0)])
        );
    }

    #[test]
    fn test_check_win_vertical() {
        let mut game = TicTacToe::new();

        game._make_move((0, 0), Player::Player1).unwrap();
        game._make_move((0, 1), Player::Player1).unwrap();
        game._make_move((0, 2), Player::Player1).unwrap();

        assert_eq!(
            game.check_win((0, 0)),
            Some(smallvec![(0, 0), (0, 1), (0, 2)])
        );
    }

    #[test]
    fn test_check_win_diagonal() {
        let mut game = TicTacToe::new();

        game._make_move((0, 0), Player::Player1).unwrap();
        game._make_move((1, 1), Player::Player1).unwrap();
        game._make_move((2, 2), Player::Player1).unwrap();

        assert_eq!(
            game.check_win((0, 0)),
            Some(smallvec![(0, 0), (1, 1), (2, 2)])
        );
    }

    #[test]
    fn test_check_win_anti_diagonal() {
        let mut game = TicTacToe::new();

        game._make_move((0, 2), Player::Player1).unwrap();
        game._make_move((1, 1), Player::Player1).unwrap();
        game._make_move((2, 0), Player::Player1).unwrap();

        assert_eq!(
            game.check_win((0, 2)),
            Some(smallvec![(0, 2), (1, 1), (2, 0)])
        );
    }

    #[test]
    fn test_check_no_win_horizontal() {
        let mut game = TicTacToe::new();

        game._make_move((0, 2), Player::Player1).unwrap();
        game._make_move((1, 2), Player::Player1).unwrap();
        game._make_move((2, 0), Player::Player1).unwrap();
        assert_eq!(game.check_win((0, 2)), None);
    }

    #[test]
    fn test_check_no_win_vertical() {
        let mut game = TicTacToe::new();

        game._make_move((1, 0), Player::Player1).unwrap();
        game._make_move((1, 1), Player::Player2).unwrap();
        game._make_move((1, 2), Player::Player1).unwrap();
        assert_eq!(game.check_win((1, 0)), None);
    }
    #[test]
    fn test_tie_state() {
        let mut game = TicTacToe::new();

        // Simulate a tied state
        game._make_move((0, 0), Player::Player1).unwrap();
        game._make_move((1, 0), Player::Player2).unwrap();
        game._make_move((2, 0), Player::Player1).unwrap();
        game._make_move((0, 1), Player::Player2).unwrap();
        game._make_move((0, 2), Player::Player1).unwrap();
        game._make_move((1, 1), Player::Player2).unwrap();
        game._make_move((1, 2), Player::Player1).unwrap();
        game._make_move((2, 2), Player::Player2).unwrap();
        game._make_move((2, 1), Player::Player1).unwrap();

        // Ensure the game state is set to tie
        assert!(game.check_tie((2, 2)));
    }

    #[test]
    fn test_tie_state_win() {
        let mut game = TicTacToe::new();

        // Simulate a tied state
        game._make_move((0, 0), Player::Player1).unwrap();
        game._make_move((0, 1), Player::Player2).unwrap();
        game._make_move((1, 1), Player::Player1).unwrap();
        game._make_move((0, 2), Player::Player2).unwrap();
        game._make_move((2, 0), Player::Player1).unwrap();
        game._make_move((1, 0), Player::Player2).unwrap();
        game._make_move((1, 2), Player::Player1).unwrap();
        game._make_move((2, 1), Player::Player2).unwrap();
        game._make_move((2, 2), Player::Player1).unwrap();

        assert_eq!(
            game.check_win((0, 0)),
            Some(smallvec![(0, 0), (1, 1), (2, 2)])
        );

        // this should be a win for Player 1
        assert_ne!(true, game.check_tie((2, 2)));
    }
    #[test]
    fn test_reset_after_tie() {
        let mut game = TicTacToe::new();

        game._make_move((0, 0), Player::Player1).unwrap();
        game._make_move((1, 0), Player::Player2).unwrap();
        game._make_move((2, 0), Player::Player1).unwrap();
        game._make_move((0, 1), Player::Player2).unwrap();
        game._make_move((0, 2), Player::Player1).unwrap();
        game._make_move((1, 1), Player::Player2).unwrap();
        game._make_move((1, 2), Player::Player1).unwrap();
        game._make_move((2, 2), Player::Player2).unwrap();
        game._make_move((2, 1), Player::Player1).unwrap();

        // Ensure the game is a tie
        assert!(game.check_tie((2, 2)));

        // Manually set the game state to tie
        game.state = TicTacToeState::Tie;

        // Simulate a reset
        let reset_command = GameCommand {
            player: Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Select,
        };
        game.process_input(reset_command).unwrap();

        // Ensure the game state is reset to Playing and the board is empty
        assert_eq!(game.state, TicTacToeState::Playing);
        for row in game.board.cells.iter() {
            for cell in row.iter() {
                assert_eq!(*cell, Cell::Empty);
            }
        }
    }
}
