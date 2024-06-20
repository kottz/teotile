use crate::animation::Animation;
use crate::game::{ButtonState, CommandType, Game, GameCommand, RenderBoard, Result, RGB};

use core::time::Duration;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use smallvec::SmallVec;

const GRID_SIZE: usize = 12;
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);

struct Player {
    col: usize,
    pos: f64,
    velocity: f64,
}

impl Player {
    fn new() -> Self {
        Self {
            col: 0,
            pos: 0.0,
            velocity: 0.0,
        }
    }

    fn jump(&mut self) {
        self.velocity = 0.8;
    }

    fn update_position(&mut self, delta_time: f64) {
        const GRAVITY: f64 = -0.2;
        const SPEED_MULTIPLIER: f64 = 15.0;
        const TOP_BOUNCE_VELOCITY: f64 = -0.3;

        self.pos += self.velocity * delta_time * SPEED_MULTIPLIER;
        self.velocity += GRAVITY * delta_time * SPEED_MULTIPLIER;

        if self.pos <= 0.0 {
            self.pos = 0.0;
            self.velocity = 0.0;
        } else if self.pos >= (GRID_SIZE - 1) as f64 {
            self.pos = (GRID_SIZE - 1) as f64;
            self.velocity = TOP_BOUNCE_VELOCITY;
        }
    }

    fn row(&self) -> usize {
        libm::round(self.pos) as usize
    }
}

struct Wall {
    col: usize,
    gap_row: usize,
    gap_size: usize,
}

impl Wall {
    fn new(col: usize, gap_row: usize, gap_size: usize) -> Self {
        Self {
            col,
            gap_row,
            gap_size,
        }
    }
}

enum GameState {
    Playing,
    GameOver,
}

pub struct FlappyBird {
    state: GameState,
    player: Player,
    walls: SmallVec<[Wall; GRID_SIZE]>,
    current_time: Duration,
    wall_gap: usize,
    wall_period: f64,
    last_wall_time: f64,
    rng: SmallRng,
    game_over_animation: Animation,
    score: usize,
}

impl FlappyBird {
    pub fn new(seed: u64) -> Self {
        Self {
            state: GameState::Playing,
            player: Player::new(),
            walls: SmallVec::new(),
            current_time: Duration::ZERO,
            wall_gap: 8,
            wall_period: 0.18,
            last_wall_time: 0.0,
            rng: SmallRng::seed_from_u64(seed),
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
            score: 0,
        }
    }

    fn move_walls_left(&mut self) {
        self.walls.retain_mut(|wall| {
            if wall.col == 0 {
                false
            } else {
                wall.col -= 1;
                if wall.col == self.player.col {
                    self.score += 1;
                }
                true
            }
        });
    }

    fn add_wall(&mut self) {
        const GAP_SIZE: usize = 4;
        let gap_row = self.rng.gen_range(0..=GRID_SIZE - GAP_SIZE);
        self.walls.push(Wall::new(GRID_SIZE - 1, gap_row, GAP_SIZE));
    }

    fn detect_collision(&self) -> bool {
        self.walls.iter().any(|wall| {
            wall.col == self.player.col
                && !(wall.gap_row <= self.player.row()
                    && self.player.row() < wall.gap_row + wall.gap_size)
        })
    }

    fn reset_game(&mut self) {
        self.state = GameState::Playing;
        self.walls.clear();
        self.player = Player::new();
        self.score = 0;
    }
}

impl Game for FlappyBird {
    fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
        if let ButtonState::Pressed = input_command.button_state {
            match (&self.state, input_command.command_type) {
                (GameState::Playing, CommandType::Up | CommandType::Select) => self.player.jump(),
                (GameState::GameOver, CommandType::Select) => self.reset_game(),
                _ => {}
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        self.current_time += delta_time;
        let dt = delta_time.as_secs_f64();

        match self.state {
            GameState::Playing => {
                self.player.update_position(dt);

                if self.detect_collision() {
                    self.state = GameState::GameOver;
                    return Ok(());
                }

                self.last_wall_time += dt;
                if self.last_wall_time > self.wall_period {
                    self.move_walls_left();
                    self.last_wall_time = 0.0;
                }

                if self.walls.is_empty()
                    || self
                        .walls
                        .last()
                        .map_or(false, |w| w.col == GRID_SIZE - self.wall_gap)
                {
                    self.add_wall();
                }
            }
            GameState::GameOver => {
                self.game_over_animation.update(self.current_time);
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
        let mut render_board = RenderBoard::new();

        for wall in &self.walls {
            for row in (0..wall.gap_row).chain(wall.gap_row + wall.gap_size..GRID_SIZE) {
                render_board.set(wall.col, row, RGB::new(255, 0, 0));
            }
        }

        if let GameState::GameOver = self.state {
            if let Some(first_wall) = self.walls.first() {
                if first_wall.col == 0 {
                    let color = self.game_over_animation.get_color();
                    for row in (0..first_wall.gap_row)
                        .chain(first_wall.gap_row + first_wall.gap_size..GRID_SIZE)
                    {
                        render_board.set(0, row, color);
                    }
                }
            }
        }

        let player_color = match self.state {
            GameState::Playing => RGB::new(0, 255, 0),
            GameState::GameOver => RGB::new(189, 20, 20),
        };
        render_board.set(self.player.col, self.player.row(), player_color);

        Ok(render_board)
    }
}
