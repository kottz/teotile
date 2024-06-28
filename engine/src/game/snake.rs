use crate::animation::Animation;
use crate::game::{ButtonState, CommandType, Game, GameCommand, Player};
use crate::random::CustomRng;
use crate::RGB;
use crate::{GameError, RenderBoard};
use core::time::Duration;
use smallvec::SmallVec;

const GRID_SIZE: usize = 12;
const MAX_SNAKES: usize = 2;
const UPDATE_INTERVAL: Duration = Duration::from_millis(150);
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnakeGameMode {
    SinglePlayer,
    MultiPlayer,
}

#[derive(Debug, PartialEq)]
enum GameState {
    Playing,
    GameOver(Option<Player>),
}

#[derive(Debug, Clone)]
struct Snake {
    body: SmallVec<[(usize, usize); 128]>,
    direction: (i32, i32),
    next_direction: (i32, i32),
    player: Player,
    growth_pending: usize,
    move_queued: bool,
}

impl Snake {
    fn new(player: Player, start_pos: (usize, usize)) -> Self {
        let mut body = SmallVec::new();
        body.push(start_pos);

        Self {
            body,
            direction: (1, 0),
            next_direction: (1, 0),
            player,
            growth_pending: 0,
            move_queued: false,
        }
    }

    fn head(&self) -> (usize, usize) {
        self.body[0]
    }

    fn set_direction(&mut self, new_direction: (i32, i32)) {
        if !self.move_queued
            && (new_direction.0 != -self.direction.0 || new_direction.1 != -self.direction.1)
        {
            self.next_direction = new_direction;
            self.move_queued = true;
        }
    }

    fn move_snake(&mut self) {
        self.direction = self.next_direction;
        self.move_queued = false;

        let (dx, dy) = self.direction;
        let (x, y) = self.head();

        let new_x = (x as i32 + dx).rem_euclid(GRID_SIZE as i32) as usize;
        let new_y = (y as i32 + dy).rem_euclid(GRID_SIZE as i32) as usize;

        self.body.insert(0, (new_x, new_y));

        if self.growth_pending > 0 {
            self.growth_pending -= 1;
        } else {
            self.body.pop();
        }
    }

    fn grow(&mut self) {
        self.growth_pending += 1;
    }
}

pub struct SnakeGame {
    mode: SnakeGameMode,
    state: GameState,
    snakes: SmallVec<[Snake; MAX_SNAKES]>,
    food: SmallVec<[(usize, usize); 2]>,
    current_time: Duration,
    last_update_time: Duration,
    game_over_animation: Animation,
    rng: CustomRng,
    num_food: usize,
}

impl SnakeGame {
    pub fn new(seed: u64, mode: SnakeGameMode) -> Self {
        let rng = CustomRng::seed_from_u64(seed);
        let mut snakes = SmallVec::new();
        snakes.push(Snake::new(Player::Player1, (GRID_SIZE / 4, GRID_SIZE / 2)));

        let num_food = match mode {
            SnakeGameMode::SinglePlayer => 1,
            SnakeGameMode::MultiPlayer => 2,
        };

        if mode == SnakeGameMode::MultiPlayer {
            snakes.push(Snake::new(
                Player::Player2,
                (3 * GRID_SIZE / 4, GRID_SIZE / 2),
            ));
        }

        let mut game = Self {
            mode,
            state: GameState::Playing,
            snakes,
            food: SmallVec::new(),
            current_time: Duration::ZERO,
            last_update_time: Duration::ZERO,
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
            rng,
            num_food,
        };

        game.spawn_food();
        game
    }

    fn spawn_food(&mut self) {
        while self.food.len() < self.num_food {
            let x = self.rng.gen_range(0, GRID_SIZE as u32) as usize;
            let y = self.rng.gen_range(0, GRID_SIZE as u32) as usize;

            if !self.snakes.iter().any(|snake| snake.body.contains(&(x, y)))
                && !self.food.contains(&(x, y))
            {
                self.food.push((x, y));
            }
        }
    }

    fn check_collisions(&self) -> Option<GameState> {
        for (i, snake) in self.snakes.iter().enumerate() {
            let head = snake.head();

            // Self-collision
            if snake.body[1..].contains(&head) {
                return Some(match self.mode {
                    SnakeGameMode::SinglePlayer => GameState::GameOver(None),
                    SnakeGameMode::MultiPlayer => {
                        GameState::GameOver(Some(self.snakes[1 - i].player))
                    }
                });
            }

            // Collision with other snake (multiplayer only)
            if self.mode == SnakeGameMode::MultiPlayer {
                let other_snake = &self.snakes[1 - i];
                if other_snake.body.contains(&head) {
                    return Some(GameState::GameOver(Some(other_snake.player)));
                }
            }
        }

        None
    }

    fn process_food(&mut self) {
        let mut food_eaten = false;

        // Check for food collisions and grow snakes
        for snake in &mut self.snakes {
            if let Some(food_index) = self.food.iter().position(|&f| f == snake.head()) {
                snake.grow();
                self.food.swap_remove(food_index);
                food_eaten = true;
            }
        }

        if food_eaten {
            self.spawn_food();
        }
    }
}

impl Game for SnakeGame {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        match self.state {
            GameState::Playing => {
                if let ButtonState::Pressed = input_command.button_state {
                    let direction = match input_command.command_type {
                        CommandType::Up => (0, 1),
                        CommandType::Down => (0, -1),
                        CommandType::Left => (-1, 0),
                        CommandType::Right => (1, 0),
                        _ => return Ok(()),
                    };

                    if let Some(snake) = self
                        .snakes
                        .iter_mut()
                        .find(|s| s.player == input_command.player)
                    {
                        snake.set_direction(direction);
                    }
                }
            }
            GameState::GameOver(_) => {
                if let (ButtonState::Pressed, CommandType::Select) =
                    (input_command.button_state, input_command.command_type)
                {
                    *self = SnakeGame::new(self.rng.next_u64(), self.mode);
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.current_time += delta_time;

        match self.state {
            GameState::Playing => {
                if self.current_time - self.last_update_time > UPDATE_INTERVAL {
                    self.last_update_time = self.current_time;

                    for snake in &mut self.snakes {
                        snake.move_snake();
                    }

                    self.process_food();

                    if let Some(new_state) = self.check_collisions() {
                        self.state = new_state;
                    }
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

        match &self.state {
            GameState::Playing => {
                for snake in &self.snakes {
                    let color = match snake.player {
                        Player::Player1 => RGB::new(0, 255, 0),
                        Player::Player2 => RGB::new(0, 0, 255),
                    };
                    for &(x, y) in &snake.body {
                        render_board.set(x, y, color);
                    }
                }

                for &(x, y) in &self.food {
                    render_board.set(x, y, RGB::new(255, 0, 0));
                }
            }
            GameState::GameOver(winner) => {
                let game_over_color = self.game_over_animation.get_color();

                for snake in &self.snakes {
                    let color = if Some(snake.player) != *winner {
                        game_over_color
                    } else {
                        match snake.player {
                            Player::Player1 => RGB::new(0, 255, 0),
                            Player::Player2 => RGB::new(0, 0, 255),
                        }
                    };

                    for &(x, y) in snake.body.iter().skip(1) {
                        render_board.set(x, y, color);
                    }
                }

                // Second pass: render snake heads
                // so the loss is more visible
                for snake in &self.snakes {
                    let color = if Some(snake.player) != *winner {
                        game_over_color
                    } else {
                        match snake.player {
                            Player::Player1 => RGB::new(0, 255, 0),
                            Player::Player2 => RGB::new(0, 0, 255),
                        }
                    };

                    let &(head_x, head_y) = snake.body.first().unwrap();
                    render_board.set(head_x, head_y, color);
                }
            }
        }
        Ok(render_board)
    }
}
