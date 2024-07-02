use crate::game::{Board, ButtonState, CommandType, Game, GameCommand};
use crate::RGB;
use crate::{GameError, RenderBoard};
use core::time::Duration;

const GRID_SIZE: usize = 12;
const COLOR_ROW: usize = GRID_SIZE - 1;
const CANVAS_SIZE: usize = GRID_SIZE;

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
        // No continuous updates needed for this game
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        // Render canvas
        for y in 0..COLOR_ROW {
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
        let cursor_color = self.get_cursor_color();
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
            _ => Color::Empty,
        }
    }

    fn get_cursor_color(&self) -> RGB {
        if self.cursor.1 == COLOR_ROW {
            return RGB::new(128, 128, 128); // Gray for color selection row
        }

        let underlying_color = self.board.get(self.cursor.0, self.cursor.1);
        let selected_rgb = self.selected_color.to_rgb();

        if self.selected_color == underlying_color {
            // Invert the color for contrast
            // RGB::new(
            //     255 - selected_rgb.r,
            //     255 - selected_rgb.g,
            //     255 - selected_rgb.b,
            // )
            let r = selected_rgb.r.saturating_sub(128);
            let g = selected_rgb.g.saturating_sub(128);
            let b = selected_rgb.b.saturating_sub(128);
            if r + g + b == 0 {
                RGB::new(20, 20, 20)
            } else {
                RGB::new(r, g, b)
            }
        } else if self.selected_color == Color::Empty {
            if underlying_color == Color::Empty {
                RGB::new(5, 5, 5) // Slight gray for visibility on black squares
            } else {
                RGB::new(0, 0, 0) // Black cursor on non-black squares
            }
        } else {
            selected_rgb
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ... (previous tests remain unchanged)

    #[test]
    fn test_cursor_visibility() {
        let mut game = PaintGame::new();

        // Test cursor on empty (black) square with empty selected
        game.selected_color = Color::Empty;
        game.cursor = (0, 0);
        let cursor_color = game.get_cursor_color();
        assert_eq!(cursor_color, RGB::new(5, 5, 5));

        // Test cursor on red square with red selected
        game.selected_color = Color::Red;
        game.board.set(1, 1, Color::Red);
        game.cursor = (1, 1);
        let cursor_color = game.get_cursor_color();
        assert_eq!(cursor_color, RGB::new(0, 255, 255)); // Inverted red

        // Test cursor on blue square with green selected
        game.selected_color = Color::Green;
        game.board.set(2, 2, Color::Blue);
        game.cursor = (2, 2);
        let cursor_color = game.get_cursor_color();
        assert_eq!(cursor_color, RGB::new(0, 255, 0)); // Normal green

        // Test cursor on color selection row
        game.cursor = (0, COLOR_ROW);
        let cursor_color = game.get_cursor_color();
        assert_eq!(cursor_color, RGB::new(128, 128, 128)); // Gray
    }
}
