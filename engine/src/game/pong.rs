use crate::animation::Animation;
use crate::game::{ButtonState, CommandType, Game, GameCommand, Player};
use crate::random::CustomRng;
use crate::RGB;
use crate::{GameError, RenderBoard};
use core::time::Duration;
use libm::{fabsf, roundf};

const GRID_SIZE: usize = 12;
const PLAY_AREA_HEIGHT: usize = GRID_SIZE - 1;
const PADDLE_HEIGHT: usize = 3;
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);
const BALL_SPEED: f32 = 0.1;

#[derive(Debug, PartialEq)]
enum GameState {
    Playing,
    GameOver(Player),
}

#[derive(Debug, Clone, Copy)]
struct Paddle {
    y: usize,
    score: usize,
}

#[derive(Debug, Clone, Copy)]
struct Ball {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
}

pub struct PongGame {
    state: GameState,
    paddles: [Paddle; 2],
    ball: Ball,
    rng: CustomRng,
    game_over_animation: Animation,
    game_time: Duration,
}

impl PongGame {
    pub fn new(seed: u64) -> Self {
        let mut rng = CustomRng::seed_from_u64(seed);
        Self {
            state: GameState::Playing,
            paddles: [
                Paddle {
                    y: (PLAY_AREA_HEIGHT / 2) - (PADDLE_HEIGHT / 2),
                    score: 0,
                },
                Paddle {
                    y: (PLAY_AREA_HEIGHT / 2) - (PADDLE_HEIGHT / 2),
                    score: 0,
                },
            ],
            ball: Ball {
                x: GRID_SIZE as f32 / 2.0,
                y: PLAY_AREA_HEIGHT as f32 / 2.0,
                dx: if rng.gen_bool(0.5) {
                    BALL_SPEED
                } else {
                    -BALL_SPEED
                },
                dy: rng.gen_range_f32(-BALL_SPEED, BALL_SPEED),
            },
            rng,
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
            game_time: Duration::from_secs(0),
        }
    }

    fn move_paddle(&mut self, player: Player, direction: i32) {
        let paddle = &mut self.paddles[player as usize];
        let new_y =
            (paddle.y as i32 + direction).clamp(0, (PLAY_AREA_HEIGHT - PADDLE_HEIGHT) as i32);
        paddle.y = new_y as usize;
    }

    fn update_ball(&mut self) {
        // Update ball position
        self.ball.x += self.ball.dx;
        self.ball.y += self.ball.dy;

        let ball_x = roundf(self.ball.x) as usize;
        let ball_y = roundf(self.ball.y) as usize;

        // Bounce off top and bottom walls
        if ball_y == 0 || ball_y >= PLAY_AREA_HEIGHT - 1 {
            self.ball.dy = -self.ball.dy;
            // Ensure the ball doesn't get stuck in the wall
            self.ball.y = self.ball.y.clamp(0.0, (PLAY_AREA_HEIGHT - 1) as f32);
        }

        // Check for paddle collisions
        if ball_x == 1
            && (ball_y >= self.paddles[0].y && ball_y < self.paddles[0].y + PADDLE_HEIGHT)
        {
            self.ball.dx = fabsf(self.ball.dx); // Ensure ball moves right
            self.add_spin_to_ball(self.paddles[0].y);
        } else if ball_x == GRID_SIZE - 2
            && (ball_y >= self.paddles[1].y && ball_y < self.paddles[1].y + PADDLE_HEIGHT)
        {
            self.ball.dx = -fabsf(self.ball.dx); // Ensure ball moves left
            self.add_spin_to_ball(self.paddles[1].y);
        }

        // Cap ball speed
        let max_speed = 0.5; // Adjust this value as needed
        let speed = libm::sqrtf(self.ball.dx * self.ball.dx + self.ball.dy * self.ball.dy);
        if speed > max_speed {
            self.ball.dx = (self.ball.dx / speed) * max_speed;
            self.ball.dy = (self.ball.dy / speed) * max_speed;
        }

        // Check for scoring
        if ball_x == 0 {
            self.paddles[1].score += 1;
            self.reset_ball(Player::Player2);
        } else if ball_x == GRID_SIZE - 1 {
            self.paddles[0].score += 1;
            self.reset_ball(Player::Player1);
        }

        // Check for game over
        if self.paddles[0].score >= GRID_SIZE / 2 {
            self.state = GameState::GameOver(Player::Player1);
        } else if self.paddles[1].score >= GRID_SIZE / 2 {
            self.state = GameState::GameOver(Player::Player2);
        }
    }

    fn add_spin_to_ball(&mut self, paddle_y: usize) {
        let impact_point = self.ball.y - paddle_y as f32;
        let relative_impact = impact_point / PADDLE_HEIGHT as f32;
        let spin = (relative_impact - 0.5) * 0.2; // Adjust 0.2 to control the amount of spin
        self.ball.dy += spin;
    }

    fn reset_ball(&mut self, serving_player: Player) {
        self.ball.x = GRID_SIZE as f32 / 2.0;
        self.ball.y = PLAY_AREA_HEIGHT as f32 / 2.0;
        self.ball.dx = match serving_player {
            Player::Player1 => BALL_SPEED,
            Player::Player2 => -BALL_SPEED,
        };
        self.ball.dy = self.rng.gen_range_f32(-BALL_SPEED, BALL_SPEED);
    }
}

impl Game for PongGame {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        match self.state {
            GameState::Playing => {
                if let ButtonState::Pressed = input_command.button_state {
                    match (input_command.player, input_command.command_type) {
                        (Player::Player1, CommandType::Down) => {
                            self.move_paddle(Player::Player1, -1)
                        }
                        (Player::Player1, CommandType::Up) => self.move_paddle(Player::Player1, 1),
                        (Player::Player2, CommandType::Down) => {
                            self.move_paddle(Player::Player2, -1)
                        }
                        (Player::Player2, CommandType::Up) => self.move_paddle(Player::Player2, 1),
                        _ => {}
                    }
                }
            }
            GameState::GameOver(_) => {
                if let (ButtonState::Pressed, CommandType::Select) =
                    (input_command.button_state, input_command.command_type)
                {
                    *self = Self::new(self.rng.next_u64());
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.game_time = self.game_time.saturating_add(delta_time);

        match self.state {
            GameState::Playing => {
                self.update_ball();
            }
            GameState::GameOver(_) => {
                let animation_time =
                    Duration::from_nanos((self.game_time.as_nanos() % u64::MAX as u128) as u64);
                self.game_over_animation.update(animation_time);
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        // Render the bottom row as out of play
        for x in 0..GRID_SIZE {
            render_board.set(x, GRID_SIZE - 1, RGB::new(20, 20, 20));
        }

        match self.state {
            GameState::Playing => {
                // Render paddles
                for y in 0..PADDLE_HEIGHT {
                    render_board.set(0, self.paddles[0].y + y, RGB::new(255, 255, 255));
                    render_board.set(
                        GRID_SIZE - 1,
                        self.paddles[1].y + y,
                        RGB::new(255, 255, 255),
                    );
                }

                // Render ball
                render_board.set(
                    roundf(self.ball.x) as usize,
                    roundf(self.ball.y) as usize,
                    RGB::new(255, 255, 255),
                );

                // Render scores
                for i in 0..self.paddles[0].score {
                    render_board.set(i, GRID_SIZE - 1, RGB::new(255, 0, 0));
                }
                for i in 0..self.paddles[1].score {
                    render_board.set(GRID_SIZE - 1 - i, GRID_SIZE - 1, RGB::new(0, 255, 0));
                }
            }
            GameState::GameOver(winner) => {
                let win_color = match winner {
                    Player::Player1 => RGB::new(255, 0, 0),
                    Player::Player2 => RGB::new(0, 255, 0),
                };
                let game_over_color = self.game_over_animation.get_color();
                for x in 0..GRID_SIZE {
                    for y in 0..GRID_SIZE {
                        render_board.set(
                            x,
                            y,
                            if x < GRID_SIZE / 2 {
                                win_color
                            } else {
                                game_over_color
                            },
                        );
                    }
                }
            }
        }

        Ok(render_board)
    }
}
