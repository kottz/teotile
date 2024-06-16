use crate::animation::Animation;
use crate::game::{ButtonState, CommandType, Game, GameCommand};
use crate::RenderBoard;
use crate::RGB;
use anyhow::Result;
use core::time::Duration;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use smallvec::SmallVec;

const GRID_SIZE: usize = 12;
const UPDATE_INTERVAL: Duration = Duration::from_millis(150);
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);

#[derive(Debug, PartialEq)]
enum SnakeState {
    Playing,
    GameOver,
}

struct Snake {
    body: SmallVec<[(usize, usize); GRID_SIZE]>,
    direction: (i32, i32),
    last_update_time: Duration,
    rng: SmallRng,
    move_queued: bool,
}

impl Snake {
    fn new(seed: u64) -> Self {
        let mut body = SmallVec::<[(usize, usize); GRID_SIZE]>::new();
        body.push((GRID_SIZE / 2, GRID_SIZE / 2));

        Self {
            body,
            direction: (1, 0),
            last_update_time: Duration::from_millis(0),
            rng: SmallRng::seed_from_u64(seed),
            move_queued: false,
        }
    }

    fn head(&self) -> (usize, usize) {
        self.body[0]
    }

    fn move_snake(&mut self) {
        let (dx, dy) = self.direction;
        let (x, y) = self.head();

        let mut new_x = x as i32 + dx;
        let mut new_y = y as i32 + dy;

        if new_x < 0 {
            new_x = GRID_SIZE as i32 - 1;
        }
        if new_y < 0 {
            new_y = GRID_SIZE as i32 - 1;
        }
        if new_x >= GRID_SIZE as i32 {
            new_x = 0;
        }
        if new_y >= GRID_SIZE as i32 {
            new_y = 0;
        }

        self.body.insert(0, (new_x as usize, new_y as usize));
        self.body.pop();
        self.move_queued = false;
    }
}

pub struct SnakeGame {
    state: SnakeState,
    snake: Snake,
    food: Option<(usize, usize)>,
    current_time: Duration,
    game_over_animation: Animation,
    seed: u64,
}

impl SnakeGame {
    pub fn new(seed: u64) -> Self {
        Self {
            state: SnakeState::Playing,
            snake: Snake::new(seed),
            food: None,
            current_time: Duration::from_millis(0),
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
            seed,
        }
    }

    fn spawn_food(&mut self) {
        loop {
            let x = self.snake.rng.gen_range(0..GRID_SIZE);
            let y = self.snake.rng.gen_range(0..GRID_SIZE);

            if !self.snake.body.contains(&(x, y)) {
                self.food = Some((x, y));
                break;
            }
        }
    }

    fn check_collision(&self) -> bool {
        let head = self.snake.head();
        if let Some(body) = self.snake.body.get(2..) {
            let c = body.contains(&head);
            return c;
        }
        false
    }
}

impl Game for SnakeGame {
    fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
        match &self.state {
            SnakeState::Playing => {
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Up => {
                            if self.snake.direction != (0, -1) && !self.snake.move_queued {
                                self.snake.move_queued = true;
                                self.snake.direction = (0, 1);
                            }
                        }
                        CommandType::Down => {
                            if self.snake.direction != (0, 1) && !self.snake.move_queued {
                                self.snake.move_queued = true;
                                self.snake.direction = (0, -1);
                            }
                        }
                        CommandType::Left => {
                            if self.snake.direction != (1, 0) && !self.snake.move_queued {
                                self.snake.move_queued = true;
                                self.snake.direction = (-1, 0);
                            }
                        }
                        CommandType::Right => {
                            if self.snake.direction != (-1, 0) && !self.snake.move_queued {
                                self.snake.move_queued = true;
                                self.snake.direction = (1, 0);
                            }
                        }
                        _ => {}
                    }
                }
            }
            SnakeState::GameOver => {
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Select => {
                            self.state = SnakeState::Playing;
                            self.snake = Snake::new(self.seed + 1);
                            self.food = None;
                        }
                        _ => return Ok(()),
                    }
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        self.current_time += delta_time;
        match &self.state {
            SnakeState::Playing => {
                if self.current_time - self.snake.last_update_time > UPDATE_INTERVAL {
                    self.snake.last_update_time = self.current_time;
                    self.snake.move_snake();

                    if let Some(food) = self.food {
                        if self.snake.head() == food {
                            self.snake.body.push(*self.snake.body.last().unwrap());
                            self.food = None;
                        }
                    }

                    if self.food.is_none() {
                        self.spawn_food();
                    }

                    if self.check_collision() {
                        self.state = SnakeState::GameOver;
                    }
                }
            }
            SnakeState::GameOver => {
                self.game_over_animation.update(self.current_time);
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
        let mut render_board = RenderBoard::new();
        match &self.state {
            SnakeState::Playing => {
                for &(x, y) in self.snake.body.iter() {
                    render_board.set(x, y, RGB::new(0, 255, 0));
                }

                if let Some((x, y)) = self.food {
                    render_board.set(x, y, RGB::new(255, 0, 0));
                }
            }
            SnakeState::GameOver => {
                let color = self.game_over_animation.get_color();
                for &(x, y) in self.snake.body.iter() {
                    render_board.set(x, y, color);
                }
            }
        }
        Ok(render_board)
    }
}
