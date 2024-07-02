use crate::game::{Board, ButtonState, CommandType, Game, GameCommand};
use crate::RGB;
use crate::{GameError, RenderBoard};
use core::time::Duration;

const GRID_SIZE: usize = 12;
const COLOR_ROW: usize = GRID_SIZE - 1;
const CANVAS_SIZE: usize = GRID_SIZE - 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Empty,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Orange,
    Purple,
    Brown,
    Pink,
    White,
}

impl Default for Color {
    fn default() -> Self {
        Color::Empty
    }
}

impl Color {
    fn to_rgb(&self) -> RGB {
        match self {
            Color::Empty => RGB::new(0, 0, 0),
            Color::Red => RGB::new(255, 0, 0),
            Color::Green => RGB::new(0, 255, 0),
            Color::Blue => RGB::new(0, 0, 255),
            Color::Yellow => RGB::new(255, 255, 0),
            Color::Cyan => RGB::new(0, 255, 255),
            Color::Magenta => RGB::new(255, 0, 255),
            Color::Orange => RGB::new(255, 165, 0),
            Color::Purple => RGB::new(128, 0, 128),
            Color::Brown => RGB::new(165, 42, 42),
            Color::Pink => RGB::new(255, 192, 203),
            Color::White => RGB::new(255, 255, 255),
        }
    }
}

pub struct PaintGame {
    board: Board<Color, CANVAS_SIZE, CANVAS_SIZE>,
    cursor: (usize, usize),
    selected_color: Color,
}

impl Game for PaintGame {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        if let ButtonState::Pressed = input_command.button_state {
            match input_command.command_type {
                CommandType::Left => {
                    if self.cursor.0 > 0 {
                        self.cursor.0 -= 1;
                    }
                }
                CommandType::Right => {
                    if self.cursor.0 < GRID_SIZE - 1 {
                        self.cursor.0 += 1;
                    }
                }
                CommandType::Up => {
                    if self.cursor.1 < COLOR_ROW {
                        self.cursor.1 += 1;
                    }
                }
                CommandType::Down => {
                    if self.cursor.1 > 0 {
                        self.cursor.1 -= 1;
                    }
                }
                CommandType::Select => {
                    if self.cursor.1 == COLOR_ROW {
                        self.selected_color = self.get_color_from_palette(self.cursor.0);
                    } else {
                        self.board
                            .set(self.cursor.0, self.cursor.1, self.selected_color);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn update(&mut self, _delta_time: Duration) -> Result<(), GameError> {
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        // Render canvas
        for y in 0..CANVAS_SIZE {
            for x in 0..CANVAS_SIZE {
                render_board.set(x, y, self.board.get(x, y).to_rgb());
            }
        }

        // Render color selection row
        for x in 0..GRID_SIZE {
            let color = self.get_color_from_palette(x);
            render_board.set(x, COLOR_ROW, color.to_rgb());
        }

        // Render cursor
        let cursor_color = if self.cursor.1 == COLOR_ROW {
            RGB::new(128, 128, 128) // Gray for color selection row
        } else {
            self.selected_color.to_rgb()
        };
        render_board.set(self.cursor.0, self.cursor.1, cursor_color);

        Ok(render_board)
    }
}

impl PaintGame {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            cursor: (0, 0),
            selected_color: Color::Red,
        }
    }

    fn get_color_from_palette(&self, index: usize) -> Color {
        match index {
            0 => Color::Empty,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Blue,
            4 => Color::Yellow,
            5 => Color::Cyan,
            6 => Color::Magenta,
            7 => Color::Orange,
            8 => Color::Purple,
            9 => Color::Brown,
            10 => Color::Pink,
            11 => Color::White,
            _ => Color::Empty, // Default to Empty for any out-of-bounds index
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = PaintGame::new();

        for row in 0..CANVAS_SIZE {
            for col in 0..CANVAS_SIZE {
                assert_eq!(game.board.get(col, row), Color::Empty);
            }
        }

        assert_eq!(game.cursor, (0, 0));
        assert_eq!(game.selected_color, Color::Red);
    }

    #[test]
    fn test_color_selection() {
        let mut game = PaintGame::new();

        // Move cursor to the color selection row
        for _ in 0..COLOR_ROW {
            game.process_input(GameCommand {
                player: crate::game::Player::Player1,
                button_state: ButtonState::Pressed,
                command_type: CommandType::Up,
            })
            .unwrap();
        }

        // Select blue color
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Right,
        })
        .unwrap();
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Right,
        })
        .unwrap();
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Select,
        })
        .unwrap();

        assert_eq!(game.selected_color, Color::Blue);
    }

    #[test]
    fn test_painting() {
        let mut game = PaintGame::new();

        // Paint at (0, 0)
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Select,
        })
        .unwrap();

        assert_eq!(game.board.get(0, 0), Color::Red);

        // Move to (1, 1) and paint
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Right,
        })
        .unwrap();
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Up,
        })
        .unwrap();
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Select,
        })
        .unwrap();

        assert_eq!(game.board.get(1, 1), Color::Red);
    }

    #[test]
    fn test_boundary_movement() {
        let mut game = PaintGame::new();

        // Try to move left from (0, 0)
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Left,
        })
        .unwrap();
        assert_eq!(game.cursor, (0, 0));

        // Move to top-right corner
        for _ in 0..GRID_SIZE - 1 {
            game.process_input(GameCommand {
                player: crate::game::Player::Player1,
                button_state: ButtonState::Pressed,
                command_type: CommandType::Right,
            })
            .unwrap();
        }
        for _ in 0..GRID_SIZE - 1 {
            game.process_input(GameCommand {
                player: crate::game::Player::Player1,
                button_state: ButtonState::Pressed,
                command_type: CommandType::Up,
            })
            .unwrap();
        }
        assert_eq!(game.cursor, (GRID_SIZE - 1, COLOR_ROW));

        // Try to move beyond boundaries
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Right,
        })
        .unwrap();
        game.process_input(GameCommand {
            player: crate::game::Player::Player1,
            button_state: ButtonState::Pressed,
            command_type: CommandType::Up,
        })
        .unwrap();
        assert_eq!(game.cursor, (GRID_SIZE - 1, COLOR_ROW));
    }
}
