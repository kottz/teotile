use crate::animation::Animation;
use crate::game::{ButtonState, CommandType, Game, GameCommand, Player};
use crate::{GameError, RenderBoard};
use crate::RGB;
use core::time::Duration;
use crate::random::CustomRng;
use smallvec::SmallVec;

use super::GameMode;

const GRID_SIZE: usize = 12;
const MAX_PLAYERS: usize = 2;
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);

#[derive(Debug, PartialEq)]
enum GameState {
    Playing,
    GameOver(Option<Player>),
}

#[derive(Debug, Clone, Copy)]
struct Spaceship {
    col: usize,
    player: Player,
}

impl Spaceship {
    fn new(player: Player, col: usize) -> Self {
        Self { player, col }
    }

    fn move_horizontal(&mut self, direction: isize) {
        self.col = (self.col as isize + direction).clamp(0, GRID_SIZE as isize - 1) as usize;
    }
}

#[derive(Debug, Clone, Copy)]
struct Projectile {
    row: f64,
    col: usize,
    active: bool,
    direction: isize,
    speed: f64,
}

impl Projectile {
    fn new(row: usize, col: usize, direction: isize, speed: f64) -> Self {
        Self {
            row: row as f64,
            col,
            active: true,
            direction,
            speed,
        }
    }
}

pub struct SpaceInvaders {
    mode: GameMode,
    state: GameState,
    spaceships: SmallVec<[Spaceship; MAX_PLAYERS]>,
    aliens: SmallVec<[(usize, usize); 128]>,
    projectiles: SmallVec<[Projectile; 128]>,
    current_time: Duration,
    alien_direction: isize,
    alien_move_period: f64,
    last_alien_move_time: f64,
    game_over_animation: Animation,
    walls: Option<SmallVec<[(usize, usize); 32]>>,
    difficulty: u8,
    rng: CustomRng,
}

impl SpaceInvaders {
    pub fn new(seed: u64, use_walls: bool, difficulty: u8, mode: GameMode) -> Self {
        let mut aliens = SmallVec::with_capacity(128);
        for row in 8..11 {
            for col in 2..GRID_SIZE - 2 {
                aliens.push((row, col));
            }
        }

        let walls = if use_walls {
            let mut walls = SmallVec::with_capacity(32);
            for col in [2, 5, 8, 11] {
                for row in 1..3 {
                    walls.push((row, col));
                }
            }
            Some(walls)
        } else {
            None
        };

        let mut spaceships = SmallVec::new();
        spaceships.push(Spaceship::new(Player::Player1, 0));
        if mode == GameMode::MultiPlayer {
            spaceships.push(Spaceship::new(Player::Player2, GRID_SIZE - 1));
        }

        Self {
            mode,
            state: GameState::Playing,
            spaceships,
            aliens,
            projectiles: SmallVec::with_capacity(128),
            current_time: Duration::default(),
            alien_direction: 1,
            alien_move_period: 0.8,
            last_alien_move_time: 0.0,
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
            walls,
            difficulty: difficulty.clamp(1, 5),
            rng: CustomRng::seed_from_u64(seed),
        }
    }

    fn move_spaceship(&mut self, direction: isize, player: Player) {
        if let Some(spaceship) = self.spaceships.iter_mut().find(|s| s.player == player) {
            spaceship.move_horizontal(direction);
        }
    }

    fn shoot_projectile(&mut self, player: Player) {
        const FIRE_SPEED: f64 = 10.0;
        if let Some(spaceship) = self.spaceships.iter().find(|s| s.player == player) {
            self.projectiles
                .push(Projectile::new(1, spaceship.col, 1, FIRE_SPEED));
        }
    }

    fn move_aliens(&mut self) {
        let mut change_direction = false;
        let mut lowest_alien_row = GRID_SIZE;

        for alien in &mut self.aliens {
            alien.1 = (alien.1 as isize + self.alien_direction) as usize;

            if alien.1 == 0 || alien.1 == GRID_SIZE - 1 {
                change_direction = true;
            }

            lowest_alien_row = lowest_alien_row.min(alien.0);
        }

        if change_direction {
            self.alien_direction = -self.alien_direction;

            if self.rng.gen_bool(0.8) {
                for alien in &mut self.aliens {
                    alien.0 = alien.0.saturating_sub(1);
                }
            }
        }

        if lowest_alien_row <= 1 {
            self.state = GameState::GameOver(None);
        }
    }

    fn update_projectiles(&mut self, delta_time: Duration) {
        for projectile in &mut self.projectiles {
            if projectile.active {
                let distance = projectile.speed * delta_time.as_secs_f64();
                projectile.row += distance * projectile.direction as f64;

                if projectile.row < 0.0 || projectile.row >= (GRID_SIZE - 1) as f64 {
                    projectile.active = false;
                }
            }
        }

        self.projectiles.retain(|p| p.active);
    }

    fn detect_collisions(&mut self) {
        // Alien-projectile collisions
        self.aliens.retain(|&mut (row, col)| {
            !self.projectiles.iter_mut().any(|projectile| {
                if projectile.active
                    && (libm::round(projectile.row) as usize == row)
                    && projectile.col == col
                    && projectile.direction > 0
                {
                    projectile.active = false;
                    return true;
                }
                false
            })
        });

        // Wall-projectile collisions
        if let Some(walls) = &mut self.walls {
            walls.retain(|&mut (row, col)| {
                !self.projectiles.iter_mut().any(|projectile| {
                    if projectile.active
                        && (libm::round(projectile.row) as usize == row)
                        && projectile.col == col
                    {
                        projectile.active = false;
                        return true;
                    }
                    false
                })
            });
        }

        // Remove spaceship if hit by projectile
        self.spaceships.retain(|spaceship| {
            !self.projectiles.iter_mut().any(|projectile| {
                if projectile.active
                    && (libm::round(projectile.row) as usize == 0)
                    && projectile.col == spaceship.col
                    && projectile.direction < 0
                {
                    projectile.active = false;
                    true
                } else {
                    false
                }
            })
        });

        if self.spaceships.is_empty() {
            self.state = GameState::GameOver(None);
        }

        self.projectiles.retain(|projectile| projectile.active);
    }

    fn enemy_fire(&mut self) {
        let fire_chance = self.difficulty as f32 / 100.0;
        const ALIEN_FIRE_SPEED: f64 = 5.0;
        for &(row, col) in &self.aliens {
            if self.rng.gen_bool(fire_chance as f64) {
                self.projectiles
                    .push(Projectile::new(row, col, -1, ALIEN_FIRE_SPEED));
            }
        }
    }
}

impl Game for SpaceInvaders {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        match self.state {
            GameState::Playing => {
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Left => self.move_spaceship(-1, input_command.player),
                        CommandType::Right => self.move_spaceship(1, input_command.player),
                        CommandType::Up | CommandType::Select => {
                            self.shoot_projectile(input_command.player)
                        }
                        _ => {}
                    }
                }
            }
            GameState::GameOver(_) => {
                if let (ButtonState::Pressed, CommandType::Select) =
                    (input_command.button_state, input_command.command_type)
                {
                    *self = Self::new(
                        self.rng.next_u64() + 69420,
                        self.walls.is_some(),
                        self.difficulty,
                        self.mode,
                    );
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

                self.last_alien_move_time += delta_time.as_secs_f64();

                if self.last_alien_move_time > self.alien_move_period {
                    self.move_aliens();
                    self.enemy_fire();
                    self.last_alien_move_time = 0.0;
                }

                self.detect_collisions();

                if self.aliens.is_empty() {
                    self.state = GameState::GameOver(None);
                }
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
                for &(row, col) in &self.aliens {
                    render_board.set(col, row, RGB::new(255, 0, 0));
                }

                for projectile in &self.projectiles {
                    if projectile.active {
                        render_board.set(
                            projectile.col,
                            libm::round(projectile.row) as usize,
                            RGB::new(255, 255, 255),
                        );
                    }
                }

                if let Some(walls) = &self.walls {
                    for &(row, col) in walls {
                        render_board.set(col, row, RGB::new(0, 0, 255));
                    }
                }

                for spaceship in self.spaceships.iter() {
                    let color = if spaceship.player == Player::Player1 {
                        RGB::new(0, 255, 0)
                    } else {
                        RGB::new(0, 255, 255)
                    };
                    render_board.set(spaceship.col, 0, color);
                }
            }
            GameState::GameOver(winner) => {
                let game_over_color = self.game_over_animation.get_color();
                for &(row, col) in &self.aliens {
                    render_board.set(col, row, game_over_color);
                }
                for spaceship in &self.spaceships {
                    let color = if Some(spaceship.player) == winner {
                        match spaceship.player {
                            Player::Player1 => RGB::new(0, 255, 0),
                            Player::Player2 => RGB::new(0, 255, 255),
                        }
                    } else {
                        RGB::new(255, 0, 0)
                    };
                    render_board.set(spaceship.col, 0, color);
                }
            }
        }

        Ok(render_board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spaceship_movement() {
        let mut game = SpaceInvaders::new(0, false, 3);
        assert_eq!(game.spaceship.col, 0);

        game.move_spaceship(1);
        assert_eq!(game.spaceship.col, 1);

        game.move_spaceship(-1);
        assert_eq!(game.spaceship.col, 0);

        // Test boundary conditions
        for _ in 0..GRID_SIZE {
            game.move_spaceship(1);
        }
        assert_eq!(game.spaceship.col, GRID_SIZE - 1);

        for _ in 0..GRID_SIZE {
            game.move_spaceship(-1);
        }
        assert_eq!(game.spaceship.col, 0);
    }

    #[test]
    fn test_shoot_projectile() {
        let mut game = SpaceInvaders::new(0, false, 3);
        game.shoot_projectile();
        assert_eq!(game.projectiles.len(), 1);
        assert_eq!(game.projectiles[0].row, 1.0);
        assert_eq!(game.projectiles[0].col, game.spaceship.col);
        assert!(game.projectiles[0].active);
    }

    #[test]
    fn test_update_projectiles() {
        let mut game = SpaceInvaders::new(0, false, 3);
        game.shoot_projectile();
        game.update_projectiles(Duration::from_secs_f32(0.1));
        assert!(game.projectiles[0].row > 1.0);
        assert!(game.projectiles[0].active);

        // Move projectile out of bounds
        for _ in 0..100 {
            game.update_projectiles(Duration::from_secs_f32(0.1));
        }
        assert_eq!(game.projectiles.len(), 0);
    }
}
