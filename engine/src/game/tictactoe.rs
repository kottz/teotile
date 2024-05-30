use crate::game::{Board, ButtonState, Cell, CommandType, Game, GameCommand};
use crate::RenderBoard;
use crate::RGB;
use anyhow::Result;
use core::time::Duration;
use libm::{fabs, sin};
use smallvec::SmallVec;

use super::Player;

const WIN_ANIMATION_SPEED: Duration = Duration::from_millis(50);

#[derive(Debug, PartialEq)]
pub enum TicTacToeState {
    Playing,
    Win(SmallVec<[(usize, usize); 3]>),
    Draw,
    Finished,
}

#[derive(Debug, PartialEq)]
struct WinAnimationState {
    state: usize,
    last_update_time: Duration,
}

type TicTacToeBoard = Board<Cell, 3, 3>;

impl TicTacToeBoard {
    fn check_draw(&self) -> bool {
        self.cells.iter().flatten().all(|&cell| cell != Cell::Empty)
    }
}

pub struct TicTacToe {
    pub board: TicTacToeBoard,
    active_player: Player,
    active_cell: (usize, usize),
    pub state: TicTacToeState,
    win_animation_state: WinAnimationState,
    current_time: Duration,
}

impl Game for TicTacToe {
    fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
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
                            if self.board.check_draw() {
                                self.state = TicTacToeState::Draw;
                            }
                        }
                        CommandType::Quit => {
                            self.state = TicTacToeState::Finished;
                        }
                    }
                }
            }
            TicTacToeState::Win(_) | TicTacToeState::Draw => {
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

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        self.current_time += delta_time;

        match &self.state {
            TicTacToeState::Playing => {
                self.win_animation_state.last_update_time = self.current_time;
            }
            TicTacToeState::Win(_) | TicTacToeState::Draw => {
                if self.current_time - self.win_animation_state.last_update_time
                    > WIN_ANIMATION_SPEED
                {
                    self.win_animation_state.last_update_time = self.current_time;

                    if self.win_animation_state.state >= 20 {
                        self.win_animation_state.state = 0;
                    } else {
                        self.win_animation_state.state += 1;
                    }
                }
            }
            TicTacToeState::Finished => {}
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
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
                let s = self.win_animation_state.state;
                let f: f64 = s as f64;
                let s = fabs(sin(f * 2.0 * 3.141 / 20.0)) * 10.0 + 10.0;
                let color = RGB::new(s as u8 * 10, s as u8 * 10, s as u8 * 10);
                for (col, row) in winning_line {
                    match self.active_player {
                        Player::Player1 => {
                            draw_x(row * 3 + 1, col * 3 + 1, &mut render_board, color);
                        }
                        Player::Player2 => {
                            draw_o(row * 3 + 1, col * 3 + 1, &mut render_board, color);
                        }
                    }
                    //render_board.set(col * 3, row * 3, color);
                }
            }
            TicTacToeState::Draw => {
                let s = self.win_animation_state.state;
                let f: f64 = s as f64;
                let s = fabs(sin(f * 2.0 * 3.141 / 20.0)) * 10.0 + 10.0;
                let color = RGB::new(s as u8 * 10, s as u8 * 10, s as u8 * 10);
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
            board: TicTacToeBoard::new(), // 3x3 grid for Tic Tac Toe
            active_player: Player::Player1,
            active_cell: (0, 0),
            state: TicTacToeState::Playing,
            win_animation_state: WinAnimationState {
                state: 0,
                last_update_time: Duration::from_millis(0),
            },
            current_time: Duration::from_millis(0),
        }
    }

    pub fn _get_cell_from_player(&self, player: Player) -> Cell {
        match player {
            Player::Player1 => Cell::PlayerX,
            Player::Player2 => Cell::PlayerO,
        }
    }

    fn _make_move(&mut self, cell: (usize, usize), player: Player) -> Result<()> {
        if self.board.get(cell.0, cell.1) != Cell::Empty {
            return Err(anyhow::anyhow!("Cell is already occupied"));
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
    fn test_draw_state() {
        let mut game = TicTacToe::new();

        // Simulate a draw state
        game._make_move((0, 0), Player::Player1).unwrap();
        game._make_move((0, 1), Player::Player2).unwrap();
        game._make_move((0, 2), Player::Player1).unwrap();
        game._make_move((1, 0), Player::Player2).unwrap();
        game._make_move((1, 1), Player::Player1).unwrap();
        game._make_move((1, 2), Player::Player2).unwrap();
        game._make_move((2, 0), Player::Player1).unwrap();
        game._make_move((2, 1), Player::Player2).unwrap();
        game._make_move((2, 2), Player::Player1).unwrap();

        // Ensure the game state is set to Draw
        assert!(game.board.check_draw());
    }

    #[test]
    fn test_reset_after_draw() {
        let mut game = TicTacToe::new();

        // Simulate a draw state
        game._make_move((0, 0), Player::Player1).unwrap();
        game._make_move((0, 1), Player::Player2).unwrap();
        game._make_move((0, 2), Player::Player1).unwrap();
        game._make_move((1, 0), Player::Player2).unwrap();
        game._make_move((1, 1), Player::Player1).unwrap();
        game._make_move((1, 2), Player::Player2).unwrap();
        game._make_move((2, 0), Player::Player1).unwrap();
        game._make_move((2, 1), Player::Player2).unwrap();
        game._make_move((2, 2), Player::Player1).unwrap();

        // Ensure the game is a draw
        assert!(game.board.check_draw());

        // Manually set the game state to Draw
        game.state = TicTacToeState::Draw;

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
