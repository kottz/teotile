use crate::game::{ButtonState, CommandType, Game, GameCommand, RenderBoard, Result, RGB};
use core::time::Duration;
use smallvec::SmallVec;

const GRID_SIZE: usize = 12;
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);
use crate::animation::Animation;

#[derive(Default)]
struct Spaceship {
    col: usize,
}

impl Spaceship {
    fn move_left(&mut self) {
        self.col = self.col.saturating_sub(1);
    }

    fn move_right(&mut self) {
        self.col = (self.col + 1).min(GRID_SIZE - 1);
    }
}

#[derive(Clone, Copy, Default)]
struct Alien {
    row: usize,
    col: usize,
}

#[derive(Clone, Copy)]
struct Projectile {
    row: usize,
    col: usize,
    active: bool,
}

impl Projectile {
    fn new(row: usize, col: usize) -> Self {
        Self {
            row,
            col,
            active: true,
        }
    }

    fn update_position(&mut self) {
        if self.row < GRID_SIZE - 1 {
            self.row += 1;
        } else {
            self.active = false;
        }
    }
}

pub struct SpaceInvaders {
    state: GameState,
    spaceship: Spaceship,
    aliens: SmallVec<[Alien; 128]>,
    projectiles: SmallVec<[Projectile; 128]>,
    current_time: Duration,
    alien_direction: isize,
    alien_move_period: f64,
    last_alien_move_time: f64,
    game_over_animation: Animation,
}

enum GameState {
    Playing,
    GameOver,
}

impl SpaceInvaders {
    pub fn new() -> Self {
        let mut aliens = SmallVec::with_capacity(128);

        for row in 6..6 + GRID_SIZE / 2 {
            for col in 2..GRID_SIZE - 1 {
                aliens.push(Alien { row, col });
            }
        }

        Self {
            state: GameState::Playing,
            spaceship: Spaceship::default(),
            aliens,
            projectiles: SmallVec::with_capacity(128),
            current_time: Duration::default(),
            alien_direction: 1,
            alien_move_period: 0.5,
            last_alien_move_time: 0.0,
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
        }
    }

    fn move_spaceship_left(&mut self) {
        self.spaceship.move_left();
    }

    fn move_spaceship_right(&mut self) {
        self.spaceship.move_right();
    }

    fn shoot_projectile(&mut self) {
        self.projectiles
            .push(Projectile::new(0, self.spaceship.col));
    }

    fn move_aliens(&mut self) {
        let mut change_direction = false;

        for alien in &mut self.aliens {
            alien.col = (alien.col as isize + self.alien_direction) as usize;

            if alien.col == 0 || alien.col == GRID_SIZE - 1 {
                change_direction = true;
            }
        }

        if change_direction {
            self.alien_direction = -self.alien_direction;

            for alien in &mut self.aliens {
                alien.row -= 1;
            }
        }

        if self.aliens.iter().any(|alien| alien.row == 0) {
            self.state = GameState::GameOver;
        }
    }

    fn update_projectiles(&mut self) {
        for projectile in &mut self.projectiles {
            if projectile.active {
                projectile.update_position();
            }
        }

        self.projectiles.retain(|p| p.active);
    }

    fn detect_collisions(&mut self) {
        let mut remaining_aliens = SmallVec::with_capacity(128);

        for alien in &self.aliens {
            let mut hit = false;

            for projectile in &mut self.projectiles {
                if projectile.active && projectile.row == alien.row && projectile.col == alien.col {
                    projectile.active = false;
                    hit = true;
                    break;
                }
            }

            if !hit {
                remaining_aliens.push(*alien);
            }
        }

        self.aliens = remaining_aliens;
        self.projectiles.retain(|projectile| projectile.active);
    }
}

impl Game for SpaceInvaders {
    fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
        match self.state {
            GameState::Playing => {
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Left => self.move_spaceship_left(),
                        CommandType::Right => self.move_spaceship_right(),
                        CommandType::Up | CommandType::Select => self.shoot_projectile(),
                        _ => {}
                    }
                }
            }
            GameState::GameOver => {
                if let ButtonState::Pressed = input_command.button_state {
                    if let CommandType::Select = input_command.command_type {
                        self.state = GameState::Playing;
                        self.aliens.clear();
                        self.projectiles.clear();
                        self.spaceship = Spaceship::default();

                        for row in 6..6 + GRID_SIZE / 2 {
                            for col in 2..GRID_SIZE - 1 {
                                self.aliens.push(Alien { row, col });
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        self.current_time += delta_time;

        match &self.state {
            GameState::Playing => {
                self.update_projectiles();

                self.last_alien_move_time += delta_time.as_secs_f64();

                if self.last_alien_move_time > self.alien_move_period {
                    self.move_aliens();
                    self.last_alien_move_time = 0.0;
                }

                self.detect_collisions();

                if self.aliens.is_empty() {
                    self.state = GameState::GameOver;
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

        match &self.state {
            GameState::Playing => {
                for alien in &self.aliens {
                    render_board.set(alien.col, alien.row, RGB::new(255, 0, 0));
                }

                for projectile in &self.projectiles {
                    if projectile.active {
                        render_board.set(projectile.col, projectile.row, RGB::new(255, 255, 255));
                    }
                }

                render_board.set(self.spaceship.col, 0, RGB::new(0, 255, 0));
            }
            GameState::GameOver => {
                let color = self.game_over_animation.get_color();
                for alien in &self.aliens {
                    render_board.set(alien.col, alien.row, color);
                }
                render_board.set(self.spaceship.col, 0, RGB::new(255, 0, 0));
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
        let mut game = SpaceInvaders::new();
        assert_eq!(game.spaceship.col, 0);

        game.move_spaceship_right();
        assert_eq!(game.spaceship.col, 1);

        game.move_spaceship_left();
        assert_eq!(game.spaceship.col, 0);

        // Test boundary conditions
        for _ in 0..GRID_SIZE {
            game.move_spaceship_right();
        }
        assert_eq!(game.spaceship.col, GRID_SIZE - 1);

        for _ in 0..GRID_SIZE {
            game.move_spaceship_left();
        }
        assert_eq!(game.spaceship.col, 0);
    }

    #[test]
    fn test_shoot_projectile() {
        let mut game = SpaceInvaders::new();
        game.shoot_projectile();
        assert_eq!(game.projectiles.len(), 1);
        assert_eq!(game.projectiles[0].row, 0);
        assert_eq!(game.projectiles[0].col, game.spaceship.col);
        assert!(game.projectiles[0].active);
    }

    #[test]
    fn test_update_projectiles() {
        let mut game = SpaceInvaders::new();
        game.shoot_projectile();
        game.update_projectiles();
        assert_eq!(game.projectiles[0].row, 1);
        assert!(game.projectiles[0].active);

        // Move projectile out of bounds
        for _ in 0..GRID_SIZE {
            game.update_projectiles();
        }
        assert_eq!(game.projectiles.len(), 0);
    }
}
