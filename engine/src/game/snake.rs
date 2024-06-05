use crate::game::{Board, ButtonState, CommandType, Game, GameCommand};
use crate::RGB;
use crate::{RenderBoard};
use anyhow::Result;
use core::time::Duration;
use libm::{fabs, sin};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use smallvec::SmallVec;

const GRID_SIZE: usize = 12;
const UPDATE_INTERVAL: Duration = Duration::from_millis(150);
const WIN_ANIMATION_SPEED: Duration = Duration::from_millis(50);

#[derive(Debug, PartialEq, Clone, Copy)]
enum CellType {
    Empty,
    Snake,
    Food,
}

#[derive(Debug, PartialEq)]
enum SnakeState {
    Playing,
    GameOver,
    Finished,
}

#[derive(Debug, PartialEq)]
struct GameOverAnimationState {
    state: usize,
    last_update_time: Duration,
}

impl GameOverAnimationState {
    fn new() -> Self {
        Self {
            state: 0,
            last_update_time: Duration::from_millis(0),
        }
    }
}

struct Coord {
    x: usize,
    y: usize,
}

struct Snake {
    body: SmallVec<[(usize, usize); GRID_SIZE]>,
    direction: (i32, i32),
    target_direction: (i32, i32),
    last_update_time: Duration,
    rng: SmallRng,
    move_queued: bool,
}

impl Snake {
    fn new() -> Self {
        let mut body = SmallVec::<[(usize, usize); GRID_SIZE]>::new();
        body.push((GRID_SIZE / 2, GRID_SIZE / 2));

        Self {
            body,
            direction: (1, 0),
            target_direction: (1, 0),
            last_update_time: Duration::from_millis(0),
            rng: SmallRng::seed_from_u64(42),
            move_queued: false,
        }
    }

    fn head(&self) -> (usize, usize) {
        self.body[0]
    }

    fn move_snake(&mut self) {
        let (dx, dy) = self.direction;
        //let (mut x, mut y) = self.head();
        let (x, y) = self.head();

        let mut new_x = (x as i32 + dx); //% GRID_SIZE as i32; //as usize;
        let mut new_y = (y as i32 + dy); //% GRID_SIZE as i32; //as usize;

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

        //x = x.wrapping_sub(GRID_SIZE) % GRID_SIZE;
        //x = x % 12;
        //y = y % 12;

        //y = y.wrapping_sub(GRID_SIZE) % GRID_SIZE;

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
    game_over_animation_state: GameOverAnimationState,
}

impl SnakeGame {
    pub fn new() -> Self {
        Self {
            state: SnakeState::Playing,
            snake: Snake::new(),
            food: None,
            current_time: Duration::from_millis(0),
            game_over_animation_state: GameOverAnimationState::new(),
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
        //self.snake.body[1..].contains(&head)
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
                            self.snake = Snake::new();
                            self.food = None;
                        }
                        _ => return Ok(()),
                    }
                }
            }
            SnakeState::Finished => {}
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        self.current_time += delta_time;
        match &self.state {
            SnakeState::Playing => {
                if self.current_time - self.snake.last_update_time > UPDATE_INTERVAL {
                    self.snake.last_update_time = self.current_time;
                    //self.snake.direction = self.snake.target_direction;
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
                if self.current_time - self.game_over_animation_state.last_update_time
                    > WIN_ANIMATION_SPEED
                {
                    self.game_over_animation_state.last_update_time = self.current_time;

                    if self.game_over_animation_state.state >= 20 {
                        self.game_over_animation_state.state = 0;
                    } else {
                        self.game_over_animation_state.state += 1;
                    }
                }
            }
            SnakeState::Finished => {}
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
                let s = self.game_over_animation_state.state;
                let f: f64 = s as f64;
                let s = fabs(sin(f * 2.0 * 3.141 / 20.0)) * 10.0 + 10.0;
                let color = RGB::new(s as u8 * 10, s as u8 * 10, s as u8 * 10);

                for &(x, y) in self.snake.body.iter() {
                    render_board.set(x, y, color);
                }
            }
            SnakeState::Finished => {}
        }

        Ok(render_board)
    }
}
