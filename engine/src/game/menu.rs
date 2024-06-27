use crate::game::{ButtonState, CommandType, Game, GameCommand};
use crate::{GameError, RenderBoard};
use crate::RGB;
use core::time::Duration;

use crate::game::ConnectFour;
use crate::game::DoodleJump;
use crate::game::FlappyBird;
use crate::game::GameMode;
use crate::game::SpaceInvaders;
use crate::game::TetrisGame;
use crate::game::TicTacToe;
use crate::game::{MazeGame, MazeGameMode};
use crate::game::{SnakeGame, SnakeGameMode};
use crate::game::MultiplayerShooter;
use crate::game::PongGame;

use crate::pixel_art;

use crate::GRID_SIZE;

const NUM_GAMES: usize = 14;

enum MenuState {
    Selecting,
    RunningGame(GameType),
}

pub struct Menu {
    active_game_index: usize,
    state: MenuState,
    current_time: Duration,
}

macro_rules! define_game_type_and_impl {
    ($($variant:ident($game:ty)),+ $(,)?) => {
        enum GameTypeInfo {
            $($variant),+
        }
        #[allow(dead_code)]
        enum GameType {
            $($variant($game)),+
        }

        impl Game for GameType {
            fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
                match self {
                    $(GameType::$variant(game) => game.process_input(input_command)),+
                }
            }

            fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
                match self {
                    $(GameType::$variant(game) => game.update(delta_time)),+
                }
            }

            fn render(&self) -> Result<RenderBoard, GameError> {
                match self {
                    $(GameType::$variant(game) => game.render()),+
                }
            }
        }
    };
}

define_game_type_and_impl!(
    ConnectFour(ConnectFour),
    TicTacToe(TicTacToe),
    FlappyBird(FlappyBird),
    Snake(SnakeGame),
    SnakeMultiPlayer(SnakeGame),
    Maze(MazeGame),
    MazeFlashLight(MazeGame),
    MazeFlashLightMultiplayer(MazeGame),
    SpaceInvaders(SpaceInvaders),
    SpaceInvadersMultiPlayer(SpaceInvaders),
    DoodleJump(DoodleJump),
    Tetris(TetrisGame),
    MultiplayerShooter(MultiplayerShooter),
    PongGame(PongGame),
);

type PixelArtImage = [[RGB; 8]; 8];

impl GameTypeInfo {
    fn pixel_art(&self) -> PixelArtImage {
        let image = match self {
            GameTypeInfo::ConnectFour => pixel_art::CONNECT_FOUR,
            GameTypeInfo::TicTacToe => pixel_art::TICTACTOE,
            GameTypeInfo::FlappyBird => pixel_art::FLAPPY_BIRD,
            GameTypeInfo::Snake => pixel_art::SNAKE,
            GameTypeInfo::SnakeMultiPlayer => pixel_art::SNAKE_MULTIPLAYER,
            GameTypeInfo::Maze => pixel_art::MAZE,
            GameTypeInfo::MazeFlashLight => pixel_art::MAZE_FLASHLIGHT,
            GameTypeInfo::MazeFlashLightMultiplayer => pixel_art::MAZE_FLASHLIGHT_MULTIPLAYER,
            GameTypeInfo::SpaceInvaders => pixel_art::SPACE_INVADERS,
            GameTypeInfo::SpaceInvadersMultiPlayer => pixel_art::SPACE_INVADERS_MULTIPLAYER,
            GameTypeInfo::DoodleJump => pixel_art::DOODLE_JUMP,
            GameTypeInfo::Tetris => pixel_art::TETRIS,
            GameTypeInfo::MultiplayerShooter => pixel_art::SHOOTER,
            GameTypeInfo::PongGame => pixel_art::PONG,
        };
        let mut pixel_art = [[RGB::default(); 8]; 8];

        // Rotate image 90 degrees clockwise
        // to workaround build.rs limitation
        for i in 0..8 {
            for j in (0..8).rev() {
                let index = i * 8 + j;
                pixel_art[j][7 - i] = RGB {
                    r: image[index][0],
                    g: image[index][1],
                    b: image[index][2],
                };
            }
        }
        pixel_art
    }
}

impl Menu {
    pub fn new() -> Self {
        Self {
            active_game_index: 0,
            state: MenuState::Selecting,
            current_time: Duration::from_millis(0),
        }
    }

    fn cycle_left(&mut self) {
        if self.active_game_index > 0 {
            self.active_game_index -= 1;
        } else {
            self.active_game_index = NUM_GAMES - 1;
        }
    }

    fn cycle_right(&mut self) {
        if self.active_game_index < NUM_GAMES - 1 {
            self.active_game_index += 1;
        } else {
            self.active_game_index = 0;
        }
    }

    fn get_game_type_from_index(&self) -> GameTypeInfo {
        match self.active_game_index {
            0 => GameTypeInfo::ConnectFour,
            1 => GameTypeInfo::TicTacToe,
            2 => GameTypeInfo::FlappyBird,
            3 => GameTypeInfo::Snake,
            4 => GameTypeInfo::SnakeMultiPlayer,
            5 => GameTypeInfo::Maze,
            6 => GameTypeInfo::MazeFlashLight,
            7 => GameTypeInfo::MazeFlashLightMultiplayer,
            8 => GameTypeInfo::SpaceInvaders,
            9 => GameTypeInfo::SpaceInvadersMultiPlayer,
            10 => GameTypeInfo::DoodleJump,
            11 => GameTypeInfo::Tetris,
            12 => GameTypeInfo::MultiplayerShooter,
            13 => GameTypeInfo::PongGame,
            _ => unreachable!(),
        }
    }

    fn pixel_art(&self) -> PixelArtImage {
        self.get_game_type_from_index().pixel_art()
    }

    fn start_game(&mut self) {
        let game_type = self.get_game_type_from_index();
        let seed = self.current_time.as_millis() as u64;
        let game = match game_type {
            GameTypeInfo::ConnectFour => GameType::ConnectFour(ConnectFour::new()),
            GameTypeInfo::TicTacToe => GameType::TicTacToe(TicTacToe::new()),
            GameTypeInfo::FlappyBird => GameType::FlappyBird(FlappyBird::new(seed)),
            GameTypeInfo::Snake => {
                GameType::Snake(SnakeGame::new(seed, SnakeGameMode::SinglePlayer))
            }
            GameTypeInfo::SnakeMultiPlayer => {
                GameType::Snake(SnakeGame::new(seed, SnakeGameMode::MultiPlayer))
            }
            GameTypeInfo::Maze => GameType::Maze(MazeGame::new(seed, MazeGameMode::Normal)),
            GameTypeInfo::MazeFlashLight => {
                GameType::Maze(MazeGame::new(seed, MazeGameMode::FlashLight))
            }
            GameTypeInfo::MazeFlashLightMultiplayer => {
                GameType::Maze(MazeGame::new(seed, MazeGameMode::FlashLightMultiplayer))
            }
            GameTypeInfo::SpaceInvaders => {
                GameType::SpaceInvaders(SpaceInvaders::new(seed, true, 2, GameMode::SinglePlayer))
            }
            GameTypeInfo::SpaceInvadersMultiPlayer => {
                GameType::SpaceInvaders(SpaceInvaders::new(seed, false, 4, GameMode::MultiPlayer))
            }
            GameTypeInfo::DoodleJump => GameType::DoodleJump(DoodleJump::new(seed)),
            GameTypeInfo::Tetris => GameType::Tetris(TetrisGame::new(seed)),
            GameTypeInfo::MultiplayerShooter => GameType::MultiplayerShooter(MultiplayerShooter::new(seed, 10)),
            GameTypeInfo::PongGame => GameType::PongGame(PongGame::new(seed)),
        };
        self.state = MenuState::RunningGame(game);
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for Menu {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        match &mut self.state {
            MenuState::Selecting => {
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Left => {
                            self.cycle_left();
                        }
                        CommandType::Right => {
                            self.cycle_right();
                        }
                        CommandType::Select => {
                            self.start_game();
                        }
                        _ => {}
                    }
                }
            }
            MenuState::RunningGame(game_state) => {
                game_state.process_input(input_command)?;
                if let ButtonState::Pressed = input_command.button_state {
                    if input_command.command_type == CommandType::Quit {
                        self.state = MenuState::Selecting;
                    }
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        match &mut self.state {
            MenuState::Selecting => {
                self.current_time += delta_time;
            }
            MenuState::RunningGame(game_state) => {
                game_state.update(delta_time)?;
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();
        match &self.state {
            MenuState::Selecting => {
                //render menu items
                for i in 0..NUM_GAMES {
                    let rgb = if i == self.active_game_index {
                        RGB::new(255, 255, 255)
                    } else {
                        RGB::new(20, 20, 20)
                    };
                    render_board.set(i % GRID_SIZE, i / GRID_SIZE, rgb);
                }

                //render pixel art
                for (i, row) in self.pixel_art().iter().enumerate() {
                    for (j, &pixel) in row.iter().enumerate() {
                        render_board.set(i + 2, j + 2, pixel);
                    }
                }
            }
            MenuState::RunningGame(game_state) => {
                render_board = game_state.render()?;
            }
        }
        Ok(render_board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Player;

    #[test]
    fn test_new_main_menu() {
        let menu = Menu::new();
        assert_eq!(menu.active_game_index, 0);
        assert!(matches!(menu.state, MenuState::Selecting));
    }

    #[test]
    fn test_cycle_left() {
        let mut menu = Menu::new();
        menu.active_game_index = 1;
        menu.cycle_left();
        assert_eq!(menu.active_game_index, 0);

        menu.active_game_index = 0;
        menu.cycle_left();
        assert_eq!(menu.active_game_index, NUM_GAMES - 1);
    }

    #[test]
    fn test_cycle_right() {
        let mut menu = Menu::new();
        menu.active_game_index = 0;
        menu.cycle_right();
        assert_eq!(menu.active_game_index, 1);

        menu.active_game_index = NUM_GAMES - 1;
        menu.cycle_right();
        assert_eq!(menu.active_game_index, 0);
    }

    #[test]
    fn test_start_game() {
        let mut menu = Menu::new();
        menu.active_game_index = 0;
        menu.start_game();
        assert!(matches!(
            menu.state,
            MenuState::RunningGame(GameType::ConnectFour(_))
        ));

        menu.active_game_index = 1;
        menu.start_game();
        assert!(matches!(
            menu.state,
            MenuState::RunningGame(GameType::TicTacToe(_))
        ));
    }

    #[test]
    fn test_process_input_selecting() {
        let mut menu = Menu::new();
        let left_command = GameCommand {
            command_type: CommandType::Left,
            button_state: ButtonState::Pressed,
            player: Player::Player1,
        };
        let right_command = GameCommand {
            command_type: CommandType::Right,
            button_state: ButtonState::Pressed,
            player: Player::Player1,
        };
        let select_command = GameCommand {
            command_type: CommandType::Select,
            button_state: ButtonState::Pressed,
            player: Player::Player1,
        };

        menu.process_input(left_command).unwrap();
        assert_eq!(menu.active_game_index, NUM_GAMES - 1);

        menu.process_input(right_command).unwrap();
        assert_eq!(menu.active_game_index, 0);

        menu.process_input(select_command).unwrap();
        assert!(matches!(menu.state, MenuState::RunningGame(_)));
    }

    #[test]
    fn test_process_input_running_game() {
        let mut menu = Menu::new();
        menu.start_game();

        let quit_command = GameCommand {
            command_type: CommandType::Quit,
            button_state: ButtonState::Pressed,
            player: Player::Player1,
        };

        menu.process_input(quit_command).unwrap();
        assert!(matches!(menu.state, MenuState::Selecting));
    }

    #[test]
    fn test_render_selecting() {
        let menu = Menu::new();
        let render_board = menu.render().unwrap();

        for i in 0..NUM_GAMES {
            let expected_rgb = if i == menu.active_game_index {
                RGB::new(255, 255, 255)
            } else {
                RGB::new(100, 100, 100)
            };
            assert_eq!(render_board.get(i % GRID_SIZE, i / GRID_SIZE), expected_rgb);
        }
    }
}
