use crate::animation::Animation;
use crate::game::{ButtonState, CommandType, Game, GameCommand};
use crate::RGB;
use crate::{GameError, RenderBoard};
use core::time::Duration;
//use rand::{rngs::SmallRng, Rng, SeedableRng};
use crate::random::CustomRng;

const GRID_WIDTH: usize = 12;
const GRID_HEIGHT: usize = 12;
const UPDATE_INTERVAL: Duration = Duration::from_millis(500);
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq)]
enum TetriminoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

#[derive(Debug, Clone)]
struct Tetrimino {
    tetrimino_type: TetriminoType,
    position: (i32, i32),
    rotation: usize,
}

impl Tetrimino {
    fn new(tetrimino_type: TetriminoType) -> Self {
        Self {
            tetrimino_type,
            position: (GRID_WIDTH as i32 / 2 - 1, GRID_HEIGHT as i32 - 1),
            rotation: 0,
        }
    }

    fn get_blocks(&self) -> [(i32, i32); 4] {
        let base_blocks = match self.tetrimino_type {
            TetriminoType::I => [(0, 0), (-1, 0), (1, 0), (2, 0)],
            TetriminoType::O => [(0, 0), (1, 0), (0, -1), (1, -1)],
            TetriminoType::T => [(0, 0), (-1, 0), (1, 0), (0, -1)],
            TetriminoType::S => [(0, 0), (-1, 0), (0, -1), (1, -1)],
            TetriminoType::Z => [(0, 0), (1, 0), (0, -1), (-1, -1)],
            TetriminoType::J => [(0, 0), (-1, 0), (1, 0), (-1, -1)],
            TetriminoType::L => [(0, 0), (-1, 0), (1, 0), (1, -1)],
        };

        let (x, y) = self.position;
        base_blocks.map(|(bx, by)| {
            let (rx, ry) = self.rotate(bx, by);
            (x + rx, y + ry)
        })
    }

    fn rotate(&self, x: i32, y: i32) -> (i32, i32) {
        match self.rotation {
            0 => (x, y),
            1 => (y, -x),
            2 => (-x, -y),
            3 => (-y, x),
            _ => unreachable!(),
        }
    }

    fn move_by(&mut self, dx: i32, dy: i32) {
        self.position.0 += dx;
        self.position.1 += dy;
    }

    fn rotate_clockwise(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    fn try_rotate_clockwise(&mut self, grid: &[[Option<RGB>; GRID_WIDTH]; GRID_HEIGHT]) -> bool {
        let original_rotation = self.rotation;
        let original_position = self.position;

        self.rotate_clockwise();

        if self.is_valid_position(grid) {
            return true;
        }

        // Try wall kicks
        let kicks = [(0, 0), (-1, 0), (1, 0), (0, 1), (-1, 1), (1, 1)];
        for &(dx, dy) in &kicks {
            self.position.0 += dx;
            self.position.1 += dy;
            if self.is_valid_position(grid) {
                return true;
            }
            self.position.0 -= dx;
            self.position.1 -= dy;
        }

        // If no valid position found, revert rotation
        self.rotation = original_rotation;
        self.position = original_position;
        false
    }

    fn is_valid_position(&self, grid: &[[Option<RGB>; GRID_WIDTH]; GRID_HEIGHT]) -> bool {
        self.get_blocks().iter().all(|&(x, y)| {
            x >= 0
                && x < GRID_WIDTH as i32
                && y >= 0
                && y < GRID_HEIGHT as i32
                && grid[y as usize][x as usize].is_none()
        })
    }
}

#[derive(Debug, PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

pub struct TetrisGame {
    state: GameState,
    grid: [[Option<RGB>; GRID_WIDTH]; GRID_HEIGHT],
    current_tetrimino: Tetrimino,
    next_tetrimino: Tetrimino,
    current_time: Duration,
    last_update_time: Duration,
    game_over_animation: Animation,
    rng: CustomRng,
    score: usize,
}

impl TetrisGame {
    pub fn new(seed: u64) -> Self {
        let mut rng = CustomRng::seed_from_u64(seed);
        Self {
            state: GameState::Playing,
            grid: [[None; GRID_WIDTH]; GRID_HEIGHT],
            current_tetrimino: Tetrimino::new(Self::random_tetrimino(&mut rng)),
            next_tetrimino: Tetrimino::new(Self::random_tetrimino(&mut rng)),
            current_time: Duration::ZERO,
            last_update_time: Duration::ZERO,
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
            rng,
            score: 0,
        }
    }

    fn random_tetrimino(rng: &mut CustomRng) -> TetriminoType {
        match rng.gen_range(0, 7) {
            0 => TetriminoType::I,
            1 => TetriminoType::O,
            2 => TetriminoType::T,
            3 => TetriminoType::S,
            4 => TetriminoType::Z,
            5 => TetriminoType::J,
            6 => TetriminoType::L,
            _ => unreachable!(),
        }
    }

    fn lock_tetrimino(&mut self) {
        let color = match self.current_tetrimino.tetrimino_type {
            TetriminoType::I => RGB::new(0, 255, 255),
            TetriminoType::O => RGB::new(255, 255, 0),
            TetriminoType::T => RGB::new(128, 0, 128),
            TetriminoType::S => RGB::new(0, 255, 0),
            TetriminoType::Z => RGB::new(255, 0, 0),
            TetriminoType::J => RGB::new(0, 0, 255),
            TetriminoType::L => RGB::new(255, 128, 0),
        };

        for (x, y) in self.current_tetrimino.get_blocks() {
            self.grid[y as usize][x as usize] = Some(color);
        }

        self.clear_lines();
        self.spawn_new_tetrimino();
    }

    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;
        for y in 0..GRID_HEIGHT {
            if self.grid[y].iter().all(|cell| cell.is_some()) {
                lines_cleared += 1;
                for y2 in y..GRID_HEIGHT - 1 {
                    self.grid[y2] = self.grid[y2 + 1];
                }
                self.grid[GRID_HEIGHT - 1] = [None; GRID_WIDTH];
            }
        }

        self.score += match lines_cleared {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };
    }

    fn spawn_new_tetrimino(&mut self) {
        self.current_tetrimino = core::mem::replace(
            &mut self.next_tetrimino,
            Tetrimino::new(Self::random_tetrimino(&mut self.rng)),
        );

        if !self.is_valid_position(&self.current_tetrimino) {
            self.state = GameState::GameOver;
        }
    }

    fn move_tetrimino(&mut self, dx: i32, dy: i32) {
        let mut new_tetrimino = self.current_tetrimino.clone();
        new_tetrimino.move_by(dx, dy);
        if self.is_valid_position(&new_tetrimino) {
            self.current_tetrimino = new_tetrimino;
        }
    }

    fn hard_drop(&mut self) {
        while self.is_valid_position(&self.current_tetrimino) {
            self.current_tetrimino.move_by(0, -1);
        }
        self.current_tetrimino.move_by(0, 1);
        self.lock_tetrimino();
    }

    fn is_valid_position(&self, tetrimino: &Tetrimino) -> bool {
        tetrimino.is_valid_position(&self.grid)
    }
}

impl Game for TetrisGame {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        if let GameState::Playing = self.state {
            if let ButtonState::Pressed = input_command.button_state {
                match input_command.command_type {
                    CommandType::Left => self.move_tetrimino(-1, 0),
                    CommandType::Right => self.move_tetrimino(1, 0),
                    CommandType::Up => self.hard_drop(),
                    CommandType::Down => self.move_tetrimino(0, -1),
                    CommandType::Select => {
                        self.current_tetrimino.try_rotate_clockwise(&self.grid);
                    }
                    _ => {}
                }
            }
        } else if let (ButtonState::Pressed, CommandType::Select) =
            (input_command.button_state, input_command.command_type)
        {
            *self = TetrisGame::new(self.rng.next_u64());
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.current_time += delta_time;

        match self.state {
            GameState::Playing => {
                if self.current_time - self.last_update_time > UPDATE_INTERVAL {
                    self.last_update_time = self.current_time;

                    let mut new_tetrimino = self.current_tetrimino.clone();
                    new_tetrimino.move_by(0, -1);

                    if self.is_valid_position(&new_tetrimino) {
                        self.current_tetrimino = new_tetrimino;
                    } else {
                        self.lock_tetrimino();
                    }
                }
            }
            GameState::GameOver => {
                self.game_over_animation.update(self.current_time);
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if let Some(color) = self.grid[y][x] {
                    render_board.set(x, y, color);
                }
            }
        }

        if let GameState::Playing = self.state {
            let color = match self.current_tetrimino.tetrimino_type {
                TetriminoType::I => RGB::new(0, 255, 255),
                TetriminoType::O => RGB::new(255, 255, 0),
                TetriminoType::T => RGB::new(128, 0, 128),
                TetriminoType::S => RGB::new(0, 255, 0),
                TetriminoType::Z => RGB::new(255, 0, 0),
                TetriminoType::J => RGB::new(0, 0, 255),
                TetriminoType::L => RGB::new(255, 128, 0),
            };

            for (x, y) in self.current_tetrimino.get_blocks() {
                if x >= 0 && x < GRID_WIDTH as i32 && y >= 0 && y < GRID_HEIGHT as i32 {
                    render_board.set(x as usize, y as usize, color);
                }
            }
        }

        if let GameState::GameOver = self.state {
            let game_over_color = self.game_over_animation.get_color();
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    if self.grid[y][x].is_some() {
                        render_board.set(x, y, game_over_color);
                    }
                }
            }
        }

        Ok(render_board)
    }
}
