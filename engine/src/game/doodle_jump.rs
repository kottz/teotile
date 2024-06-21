use crate::animation::Animation;
use crate::game::{ButtonState, CommandType, Game, GameCommand, RenderBoard, Result, RGB};

use core::time::Duration;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use smallvec::SmallVec;

const GRID_SIZE: usize = 12;
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);

const GRAVITY: f64 = -0.02;
const SPEED_MULTIPLIER: f64 = 25.0;

struct Player {
    x: f64,
    y: f64,
    velocity_y: f64,
}

impl Player {
    fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            velocity_y: 0.0,
        }
    }

    fn update_position(&mut self, delta_time: f64) {
        self.velocity_y += GRAVITY * delta_time * SPEED_MULTIPLIER;
        self.y += self.velocity_y * delta_time * SPEED_MULTIPLIER;

        while self.x < 0.0 {
            self.x += GRID_SIZE as f64;
        }
        self.x %= GRID_SIZE as f64;
    }

    fn bounce(&mut self) {
        self.velocity_y = 0.5;
    }

    fn wrapped_x(&self) -> f64 {
        let mut x = self.x;
        while x < 0.0 {
            x += GRID_SIZE as f64;
        }
        x % GRID_SIZE as f64
    }

    fn col(&self) -> usize {
        libm::floor(self.wrapped_x()) as usize
    }

    fn row(&self) -> usize {
        libm::round(self.y) as usize
    }
}

struct Platform {
    x: usize,
    y: usize,
    width: usize,
}

impl Platform {
    fn new(x: usize, y: usize, width: usize) -> Self {
        Self { x, y, width }
    }
}

enum GameState {
    Playing,
    GameOver(Animation),
}

pub struct DoodleJump {
    state: GameState,
    player: Player,
    platforms: SmallVec<[Platform; GRID_SIZE * 2]>,
    current_time: Duration,
    rng: SmallRng,
    score: usize,
    camera_offset: usize,
}

impl DoodleJump {
    pub fn new(seed: u64) -> Self {
        let rng = SmallRng::seed_from_u64(seed);
        let initial_platform = Platform::new(GRID_SIZE / 2, 1, 3);

        let mut game = Self {
            state: GameState::Playing,
            player: Player::new(
                initial_platform.x as f64 + 1.0,
                initial_platform.y as f64 + 1.0,
            ),
            platforms: SmallVec::new(),
            current_time: Duration::ZERO,
            rng,
            score: 0,
            camera_offset: 0,
        };

        game.platforms.push(initial_platform);
        game.initialize_platforms();
        game
    }

    fn initialize_platforms(&mut self) {
        for y in (3..GRID_SIZE * 2).step_by(2) {
            let x = self.rng.gen_range(0..GRID_SIZE);
            let width = self.rng.gen_range(3..6);
            self.platforms.push(Platform::new(x, y, width));
        }
    }

    fn generate_platform(&mut self) {
        let x = self.rng.gen_range(0..GRID_SIZE);
        let y = self.platforms.last().map_or(GRID_SIZE * 2, |p| p.y + 2);
        let width = self.rng.gen_range(3..6);
        self.platforms.push(Platform::new(x, y, width));
    }

    fn check_and_handle_collision(&mut self) -> bool {
        let player_bottom = self.player.y;
        let player_top = self.player.y + 1.0;
        let player_x = self.player.x;

        for platform in &self.platforms {
            let platform_top = platform.y as f64 + 1.0;
            let platform_left = platform.x as f64;
            let platform_right = (platform.x + platform.width) as f64;

            let horizontal_collision = (player_x >= platform_left && player_x < platform_right)
                || ((player_x + GRID_SIZE as f64) >= platform_left
                    && (player_x + GRID_SIZE as f64) < platform_right);

            if horizontal_collision
                && player_bottom <= platform_top
                && player_top > platform_top
                && self.player.velocity_y < 0.0
            {
                self.player.y = platform_top;
                self.player.bounce();
                return true;
            }
        }
        false
    }

    fn reset_game(&mut self) {
        self.state = GameState::Playing;
        self.platforms.clear();
        let initial_platform = Platform::new(GRID_SIZE / 2, 1, 3);
        self.player = Player::new(
            initial_platform.x as f64 + 1.0,
            initial_platform.y as f64 + 1.0,
        );
        self.platforms.push(initial_platform);
        self.score = 0;
        self.camera_offset = 0;
        self.initialize_platforms();
    }
}

impl Game for DoodleJump {
    fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
        match (
            &mut self.state,
            input_command.button_state,
            input_command.command_type,
        ) {
            (GameState::Playing, ButtonState::Pressed, CommandType::Left) => self.player.x -= 1.0,
            (GameState::Playing, ButtonState::Pressed, CommandType::Right) => self.player.x += 1.0,
            (GameState::GameOver(_), ButtonState::Pressed, CommandType::Select) => {
                self.reset_game()
            }
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        self.current_time += delta_time;
        let dt = delta_time.as_secs_f64();

        match &mut self.state {
            GameState::Playing => {
                self.player.velocity_y += GRAVITY * dt * SPEED_MULTIPLIER;

                if !self.check_and_handle_collision() {
                    self.player.update_position(dt);

                    if self.player.y < self.camera_offset as f64 - 1.0 {
                        self.state = GameState::GameOver(Animation::new(GAME_OVER_ANIMATION_SPEED));
                        return Ok(());
                    }
                }

                if self.player.y > (self.camera_offset + GRID_SIZE / 2) as f64 {
                    let new_offset = self.player.row() - GRID_SIZE / 2;
                    self.score += new_offset - self.camera_offset;
                    self.camera_offset = new_offset;
                }

                self.platforms.retain(|p| p.y >= self.camera_offset);

                while self
                    .platforms
                    .last()
                    .map_or(true, |p| p.y < self.camera_offset + GRID_SIZE * 2)
                {
                    self.generate_platform();
                }
            }
            GameState::GameOver(animation) => {
                animation.update(self.current_time);
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
        let mut render_board = RenderBoard::new();

        for platform in &self.platforms {
            if platform.y >= self.camera_offset && platform.y < self.camera_offset + GRID_SIZE {
                let platform_color = match &self.state {
                    GameState::Playing => RGB::new(0, 255, 0),
                    GameState::GameOver(animation) => animation.get_color(),
                };

                for x in platform.x..platform.x + platform.width {
                    render_board.set(
                        x % GRID_SIZE,
                        platform.y - self.camera_offset,
                        platform_color,
                    );
                }
            }
        }

        let player_color = match self.state {
            GameState::Playing => RGB::new(255, 0, 0),
            GameState::GameOver(_) => RGB::new(189, 20, 20),
        };

        let player_render_x = self.player.col();
        let player_render_y = self.player.row().saturating_sub(self.camera_offset);

        if player_render_y < GRID_SIZE {
            render_board.set(player_render_x, player_render_y, player_color);
        }

        Ok(render_board)
    }
}
