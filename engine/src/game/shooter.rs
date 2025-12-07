use crate::RGB;
use crate::animation::Animation;
use crate::game::{ButtonState, CommandType, Game, GameCommand, Player};
use crate::random::CustomRng;
use crate::{GameError, RenderBoard};
use core::time::Duration;
use smallvec::SmallVec;

const GRID_SIZE: usize = 12;
const MAX_PLAYERS: usize = 2;
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);
const WALL_SPAWN_INTERVAL: f64 = 0.5; // Wall spawn interval in seconds
const MAX_WALLS: usize = 32;

#[derive(Debug, PartialEq)]
enum GameState {
    Playing,
    GameOver(Option<Player>),
}

#[derive(Debug, Clone, Copy)]
struct Character {
    row: usize,
    col: usize,
    player: Player,
}

impl Character {
    fn new(player: Player) -> Self {
        let (row, col) = match player {
            Player::Player1 => (0, GRID_SIZE / 2),
            Player::Player2 => (GRID_SIZE - 1, GRID_SIZE / 2),
        };
        Self { player, row, col }
    }

    fn move_horizontal(&mut self, direction: isize) {
        self.col = (self.col as isize + direction).clamp(0, GRID_SIZE as isize - 1) as usize;
    }
}

#[derive(Debug, Clone, Copy)]
struct Projectile {
    row: f64,
    col: f64,
    active: bool,
    direction: (f64, f64),
    speed: f64,
    player: Player,
}

impl Projectile {
    fn new(row: usize, col: usize, direction: (f64, f64), speed: f64, player: Player) -> Self {
        Self {
            row: row as f64,
            col: col as f64,
            active: true,
            direction,
            speed,
            player,
        }
    }
}

pub struct MultiplayerShooter {
    state: GameState,
    characters: SmallVec<[Character; MAX_PLAYERS]>,
    projectiles: SmallVec<[Projectile; 128]>,
    walls: SmallVec<[(usize, usize); MAX_WALLS]>,
    current_time: Duration,
    wall_spawn_timer: f64,
    game_over_animation: Animation,
    rng: CustomRng,
}

impl MultiplayerShooter {
    pub fn new(seed: u64, initial_walls: usize) -> Self {
        let mut game = Self {
            state: GameState::Playing,
            characters: SmallVec::new(),
            projectiles: SmallVec::with_capacity(128),
            walls: SmallVec::with_capacity(MAX_WALLS),
            current_time: Duration::default(),
            wall_spawn_timer: 0.0,
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
            rng: CustomRng::seed_from_u64(seed),
        };

        game.characters.push(Character::new(Player::Player1));
        game.characters.push(Character::new(Player::Player2));

        // Spawn initial walls
        for _ in 0..initial_walls.min(MAX_WALLS) {
            game.spawn_wall();
        }

        game
    }

    fn move_character(&mut self, direction: isize, player: Player) {
        if let Some(character) = self.characters.iter_mut().find(|c| c.player == player) {
            character.move_horizontal(direction);
        }
    }

    fn shoot_projectile(&mut self, player: Player) {
        const FIRE_SPEED: f64 = 10.0;
        if let Some(character) = self.characters.iter().find(|c| c.player == player) {
            let direction = match player {
                Player::Player1 => (1.0, 0.0),
                Player::Player2 => (-1.0, 0.0),
            };
            let (row, col) = match player {
                Player::Player1 => (character.row + 1, character.col),
                Player::Player2 => (character.row - 1, character.col),
            };
            self.projectiles
                .push(Projectile::new(row, col, direction, FIRE_SPEED, player));
        }
    }

    fn update_projectiles(&mut self, delta_time: Duration) {
        for projectile in &mut self.projectiles {
            if projectile.active {
                let distance = projectile.speed * delta_time.as_secs_f64();
                projectile.row += distance * projectile.direction.0;
                projectile.col += distance * projectile.direction.1;

                if projectile.row < 0.0
                    || projectile.row >= GRID_SIZE as f64
                    || projectile.col < 0.0
                    || projectile.col >= GRID_SIZE as f64
                {
                    projectile.active = false;
                }
            }
        }

        self.projectiles.retain(|p| p.active);
    }

    fn spawn_wall(&mut self) {
        if self.walls.len() < MAX_WALLS {
            let row = self.rng.gen_range(1, (GRID_SIZE - 1) as u32) as usize;
            let col = self.rng.gen_range(0, GRID_SIZE as u32) as usize;
            if !self.walls.contains(&(row, col)) {
                self.walls.push((row, col));
            }
        }
    }

    fn detect_collisions(&mut self) {
        // Wall-projectile collisions
        self.walls.retain(|&mut (wall_row, wall_col)| {
            !self.projectiles.iter_mut().any(|projectile| {
                if projectile.active
                    && (libm::round(projectile.row) as usize == wall_row)
                    && (libm::round(projectile.col) as usize == wall_col)
                {
                    projectile.active = false;
                    return true;
                }
                false
            })
        });

        // Character-projectile collisions
        for character in &self.characters {
            for projectile in &mut self.projectiles {
                if projectile.active
                    && projectile.player != character.player
                    && (libm::round(projectile.row) as usize == character.row)
                    && (libm::round(projectile.col) as usize == character.col)
                {
                    projectile.active = false;
                    self.state = GameState::GameOver(Some(projectile.player));
                    return;
                }
            }
        }

        self.projectiles.retain(|projectile| projectile.active);
    }
}

impl Game for MultiplayerShooter {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        match self.state {
            GameState::Playing => {
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Left => self.move_character(-1, input_command.player),
                        CommandType::Right => self.move_character(1, input_command.player),
                        CommandType::Select => self.shoot_projectile(input_command.player),
                        _ => {}
                    }
                }
            }
            GameState::GameOver(_) => {
                if let (ButtonState::Pressed, CommandType::Select) =
                    (input_command.button_state, input_command.command_type)
                {
                    *self = Self::new(self.rng.next_u64(), self.walls.len());
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.current_time += delta_time;

        match self.state {
            GameState::Playing => {
                self.update_projectiles(delta_time);

                self.wall_spawn_timer += delta_time.as_secs_f64();
                if self.wall_spawn_timer > WALL_SPAWN_INTERVAL {
                    self.spawn_wall();
                    self.wall_spawn_timer = 0.0;
                }

                self.detect_collisions();
            }
            GameState::GameOver(_) => {
                self.game_over_animation.update(self.current_time);
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        match self.state {
            GameState::Playing => {
                // Render characters
                for character in &self.characters {
                    let color = match character.player {
                        Player::Player1 => RGB::new(0, 255, 0),
                        Player::Player2 => RGB::new(0, 255, 255),
                    };
                    render_board.set(character.col, character.row, color);
                }

                // Render projectiles
                for projectile in &self.projectiles {
                    if projectile.active {
                        let row = libm::round(projectile.row) as usize;
                        let col = libm::round(projectile.col) as usize;
                        if row < GRID_SIZE && col < GRID_SIZE {
                            render_board.set(col, row, RGB::new(255, 255, 255));
                        }
                    }
                }

                // Render walls
                for &(row, col) in &self.walls {
                    render_board.set(col, row, RGB::new(0, 0, 255));
                }
            }
            GameState::GameOver(winner) => {
                let game_over_color = self.game_over_animation.get_color();
                for character in &self.characters {
                    let color = if Some(character.player) == winner {
                        match character.player {
                            Player::Player1 => RGB::new(0, 255, 0),
                            Player::Player2 => RGB::new(0, 255, 255),
                        }
                    } else {
                        game_over_color
                    };
                    render_board.set(character.col, character.row, color);
                }
            }
        }

        Ok(render_board)
    }
}
