use crate::animation::Animation;
use crate::game::{ButtonState, CommandType, Game, GameCommand, Player};
use crate::RGB;
use crate::{GameError, RenderBoard};
use core::time::Duration;

const GRID_SIZE: usize = 12;
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);
const COUNTDOWN_DURATION: Duration = Duration::from_secs(1);
const WIN_THRESHOLD: i32 = 12; // Lower this for a shorter game

#[derive(Debug, PartialEq)]
enum GameState {
    Countdown(CountdownState),
    Playing,
    GameOver(Player),
}

#[derive(Debug, PartialEq)]
enum CountdownState {
    Red,
    Yellow,
    Green,
}

pub struct ButtonWar {
    state: GameState,
    score_difference: i32,
    current_time: Duration,
    countdown_timer: Duration,
    game_over_animation: Animation,
}

impl ButtonWar {
    pub fn new() -> Self {
        Self {
            state: GameState::Countdown(CountdownState::Red),
            score_difference: 0,
            current_time: Duration::default(),
            countdown_timer: Duration::default(),
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
        }
    }

    fn update_countdown(&mut self, delta_time: Duration) {
        self.countdown_timer += delta_time;
        if self.countdown_timer >= COUNTDOWN_DURATION {
            self.countdown_timer = Duration::default();
            match self.state {
                GameState::Countdown(CountdownState::Red) => {
                    self.state = GameState::Countdown(CountdownState::Yellow);
                }
                GameState::Countdown(CountdownState::Yellow) => {
                    self.state = GameState::Countdown(CountdownState::Green);
                }
                GameState::Countdown(CountdownState::Green) => {
                    self.state = GameState::Playing;
                }
                _ => {}
            }
        }
    }

    fn update_score(&mut self, player: Player) {
        match player {
            Player::Player1 => self.score_difference += 1,
            Player::Player2 => self.score_difference -= 1,
        }

        if self.score_difference.abs() >= WIN_THRESHOLD {
            let winner = if self.score_difference > 0 {
                Player::Player1
            } else {
                Player::Player2
            };
            self.state = GameState::GameOver(winner);
        }
    }
}

impl Game for ButtonWar {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        if let GameState::Playing = self.state {
            if let (ButtonState::Pressed, CommandType::Select) =
                (input_command.button_state, input_command.command_type)
            {
                self.update_score(input_command.player);
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.current_time += delta_time;

        match self.state {
            GameState::Countdown(_) => self.update_countdown(delta_time),
            GameState::Playing => {}
            GameState::GameOver(_) => {
                self.game_over_animation.update(self.current_time);
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        match &self.state {
            GameState::Countdown(countdown_state) => {
                let color = match countdown_state {
                    CountdownState::Red => RGB::new(255, 0, 0),
                    CountdownState::Yellow => RGB::new(255, 255, 0),
                    CountdownState::Green => RGB::new(0, 255, 0),
                };
                // Render countdown light in the center
                for i in GRID_SIZE / 2 - 1..=GRID_SIZE / 2 + 1 {
                    for j in GRID_SIZE / 2 - 1..=GRID_SIZE / 2 + 1 {
                        render_board.set(i, j, color);
                    }
                }
            }
            GameState::Playing | GameState::GameOver(_) => {
                // Render the tug-of-war bar
                let bar_position = (GRID_SIZE as i32 / 2)
                    + (self.score_difference * GRID_SIZE as i32 / (2 * WIN_THRESHOLD));
                for row in 0..GRID_SIZE {
                    for col in 0..GRID_SIZE {
                        let color = if col as i32 == bar_position {
                            RGB::new(255, 255, 255) // White bar
                        } else if (col as i32) < bar_position {
                            RGB::new(0, 255, 0) // Green for Player 1
                        } else {
                            RGB::new(0, 255, 255) // Cyan for Player 2
                        };
                        render_board.set(col, row, color);
                    }
                }

                if let GameState::GameOver(_) = self.state {
                    // Animate the winning side
                    let winning_color = self.game_over_animation.get_color();
                    for row in 0..GRID_SIZE {
                        for col in 0..GRID_SIZE {
                            if (self.score_difference > 0 && col as i32 <= bar_position)
                                || (self.score_difference < 0 && col as i32 >= bar_position)
                            {
                                render_board.set(col, row, winning_color);
                            }
                        }
                    }
                }
            }
        }

        Ok(render_board)
    }
}
