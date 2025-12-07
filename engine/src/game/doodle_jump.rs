use crate::animation::Animation;
use crate::game::{CommandType, Game, GameCommand, RenderBoard, RGB};
use crate::random::CustomRng;
use crate::ButtonState;
use crate::GameError;

use core::time::Duration;
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

enum PlatformType {
    Static,
    Moving,
    Breaking,
    Switching,
}

struct Platform {
    x: f64,
    y: f64,
    width: usize,
    platform_type: PlatformType,
    color: RGB,
    state: PlatformState,
}

struct PlatformState {
    direction: f64,
    broken: bool,
    switch_timer: f64,
}

impl Platform {
    fn new(x: f64, y: f64, width: usize, platform_type: PlatformType) -> Self {
        let color = match platform_type {
            PlatformType::Static => RGB::new(0, 255, 0),
            PlatformType::Moving => RGB::new(0, 0, 255),
            PlatformType::Breaking => RGB::new(255, 165, 0),
            PlatformType::Switching => RGB::new(0, 255, 0),
        };

        Self {
            x,
            y,
            width,
            platform_type,
            color,
            state: PlatformState {
                direction: 1.0,
                broken: false,
                switch_timer: 0.0,
            },
        }
    }

    fn update(&mut self, dt: f64) {
        match self.platform_type {
            PlatformType::Static => {}
            PlatformType::Moving => self.update_moving(dt),
            PlatformType::Breaking => {}
            PlatformType::Switching => self.update_switching(dt),
        }
    }

    fn update_moving(&mut self, dt: f64) {
        const SPEED: f64 = 2.0;
        self.x += self.state.direction * SPEED * dt;
        if self.x <= 0.0 || self.x + self.width as f64 >= GRID_SIZE as f64 {
            self.state.direction *= -1.0;
            self.x = self.x.clamp(0.0, GRID_SIZE as f64 - self.width as f64);
        }
    }

    fn update_switching(&mut self, dt: f64) {
        const SWITCH_INTERVAL: f64 = 1.0;
        self.state.switch_timer += dt;
        if self.state.switch_timer >= SWITCH_INTERVAL {
            self.state.switch_timer = 0.0;
            self.color = if self.color.g == 255 {
                RGB::new(255, 0, 0)
            } else {
                RGB::new(0, 255, 0)
            };
        }
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
    rng: CustomRng,
    score: usize,
    camera_offset: usize,
}

impl DoodleJump {
    pub fn new(seed: u64) -> Self {
        let rng = CustomRng::seed_from_u64(seed);
        let initial_platform = Platform::new(GRID_SIZE as f64 / 2.0, 1.0, 3, PlatformType::Static);

        let mut game = Self {
            state: GameState::Playing,
            player: Player::new(initial_platform.x + 1.0, initial_platform.y + 1.0),
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
            let platform_type = self.random_platform_type();
            let width = self.rng.gen_range(3, 6);
            let max_x = GRID_SIZE as f64 - width as f64;
            let x = self.rng.gen_range_f64(0.0, max_x);
            self.platforms
                .push(Platform::new(x, y as f64, width as usize, platform_type));
        }
    }

    fn generate_platform(&mut self) {
        let y = self
            .platforms
            .last()
            .map_or(GRID_SIZE as f64 * 2.0, |p| p.y + 2.0);
        let platform_type = self.random_platform_type();
        let width = self.rng.gen_range(3, 6);

        let max_x = GRID_SIZE as f64 - width as f64;
        let x = self.rng.gen_range_f64(0.0, max_x);

        self.platforms
            .push(Platform::new(x, y, width as usize, platform_type));
    }

    fn random_platform_type(&mut self) -> PlatformType {
        match self.rng.gen_range(0, 10) {
            0..=5 => PlatformType::Static,
            6..=7 => PlatformType::Moving,
            8 => PlatformType::Breaking,
            _ => PlatformType::Switching,
        }
    }

    fn check_and_handle_collision(&mut self) -> bool {
        let player_bottom = self.player.y;
        let player_top = self.player.y + 1.0;
        let player_col = self.player.col();

        for platform in &mut self.platforms {
            let platform_top = platform.y + 1.0;
            let start_col = libm::floor(platform.x) as usize % GRID_SIZE;

            let horizontal_collision = (0..platform.width).any(|i| {
                let platform_col = (start_col + i) % GRID_SIZE;
                platform_col == player_col
            });

            let vertical_collision = player_bottom <= platform_top && player_top > platform_top;

            if horizontal_collision && vertical_collision && self.player.velocity_y < 0.0 {
                self.player.y = platform_top;
                self.player.bounce();

                match platform.platform_type {
                    PlatformType::Breaking => {
                        platform.state.broken = true;
                    }
                    PlatformType::Switching => {
                        if platform.color.g != 255 {
                            continue; // fall through if red
                        }
                    }
                    _ => {}
                }

                return true;
            }
        }
        false
    }

    fn reset_game(&mut self) {
        self.state = GameState::Playing;
        self.platforms.clear();
        let initial_platform = Platform::new(GRID_SIZE as f64 / 2.0, 1.0, 3, PlatformType::Static);
        self.player = Player::new(initial_platform.x + 1.0, initial_platform.y + 1.0);
        self.platforms.push(initial_platform);
        self.score = 0;
        self.camera_offset = 0;
        self.initialize_platforms();
    }
}

impl Game for DoodleJump {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
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

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.current_time += delta_time;
        let dt = delta_time.as_secs_f64();

        match &mut self.state {
            GameState::Playing => {
                self.player.velocity_y += GRAVITY * dt * SPEED_MULTIPLIER;

                for platform in &mut self.platforms {
                    platform.update(dt);
                }

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

                self.platforms
                    .retain(|p| p.y >= self.camera_offset as f64 && !p.state.broken);

                while self
                    .platforms
                    .last()
                    .is_none_or(|p| p.y < (self.camera_offset + GRID_SIZE * 2) as f64)
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

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        for platform in &self.platforms {
            if platform.y >= self.camera_offset as f64
                && platform.y < (self.camera_offset + GRID_SIZE) as f64
            {
                let platform_color = match &self.state {
                    GameState::Playing => platform.color,
                    GameState::GameOver(animation) => animation.get_color(),
                };

                for x in platform.x as usize..(platform.x + platform.width as f64) as usize {
                    render_board.set(
                        x % GRID_SIZE,
                        (platform.y - self.camera_offset as f64) as usize,
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
