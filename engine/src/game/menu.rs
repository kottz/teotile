use crate::game::{ButtonState, CommandType, Game, GameCommand};
use crate::RenderBoard;
use crate::RGB;
use anyhow::Result;
use core::time::Duration;

use crate::game::ConnectFour;
use crate::game::TicTacToe;

use crate::GRID_SIZE;
const NUM_GAMES: usize = 3;

enum MenuState {
    Selecting,
    RunningGame(GameType),
}

pub struct Menu {
    active_game_index: usize,
    state: MenuState,
}

enum GameType {
    ConnectFour(ConnectFour),
    TicTacToe(TicTacToe),
    //Snake,
}

impl Game for GameType {
    fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
        match self {
            GameType::ConnectFour(game) => game.process_input(input_command),
            GameType::TicTacToe(game) => game.process_input(input_command),
            // GameType::TicTacToe(game) => game.process_input(input_command),
            // GameType::Snake(game) => game.process_input(input_command),
        }
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        match self {
            GameType::ConnectFour(game) => game.update(delta_time),
            GameType::TicTacToe(game) => game.update(delta_time),
            // GameType::TicTacToe(game) => game.update(delta_time),
            // GameType::Snake(game) => game.update(delta_time),
        }
    }

    fn render(&self) -> Result<RenderBoard> {
        match self {
            GameType::ConnectFour(game) => game.render(),
            GameType::TicTacToe(game) => game.render(),
            // GameType::TicTacToe(game) => game.render(),
            // GameType::Snake(game) => game.render(),
        }
    }
}
// lazy_static! {
//     static ref GAMES: [Box<dyn Game + Sync>; 1] = [
//         &ConnectFour,
//         // &TicTacToe,
//         // &Snake,
//     ];
// }

impl Menu {
    pub fn new() -> Self {
        Self {
            //games,
            active_game_index: 0,
            state: MenuState::Selecting,
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
    // fn select_game(&mut self) {
    //     self.state = MenuState::RunningGame(self.active_game_index);
    // }
    fn select_game(&mut self) {
        let game_state = match self.active_game_index {
            0 => GameType::ConnectFour(ConnectFour::new()),
            1 => GameType::TicTacToe(TicTacToe::new()),
            // 2 => GameType::Snake(Snake::new()),
            _ => unreachable!(),
        };
        self.state = MenuState::RunningGame(game_state);
    }

    // fn process_game_input<T: Game>(&self, game: &mut T, input_command: GameCommand) -> Result<()> {
    //     game.process_input(input_command)
    // }
    //
    // fn update_game(&mut self, game_state: &mut GameType, delta_time: Duration) -> Result<()> {
    //     game_state.update(delta_time)
    // }
    //
    // fn render_game(&self, game_state: &GameType) -> Result<RenderBoard> {
    //     game_state.render()
    // }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for Menu {
    fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
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
                            self.select_game();
                        }
                        _ => {}
                    }
                }
            }
            MenuState::RunningGame(game_state) => {
                game_state.process_input(input_command)?;
                //self.process_game_input(game_state, input_command)?;
                if let ButtonState::Pressed = input_command.button_state {
                    if input_command.command_type == CommandType::Quit {
                        self.state = MenuState::Selecting;
                    }
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        if let MenuState::RunningGame(game_state) = &mut self.state {
            let _ = game_state.update(delta_time);
            //self.games[*index].update(delta_time)?;
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
        //let mut render_board = RenderBoard::new(RGB::new(0, 0, 0));
        let mut render_board = RenderBoard::new();
        match &self.state {
            MenuState::Selecting => {
                //Render main menu
                for i in 0..NUM_GAMES {
                    let rgb = if i == self.active_game_index {
                        RGB::new(255, 255, 255) // Highlight selected game
                    } else {
                        RGB::new(100, 100, 100)
                    };
                    // Display game name or some representation
                    render_board.set(i % GRID_SIZE, i / GRID_SIZE, rgb);
                }
            }
            MenuState::RunningGame(game_state) => {
                render_board = game_state.render()?;
            }
        }
        Ok(render_board)
        // if let MenuState::RunningGame(index) = &self.state {
        //     return self.games[*index].render();
        // }

        // Render main menu
        //let mut render_board = RenderBoard::new(RGB::new(0, 0, 0));
        // for (i, game) in self.games.iter().enumerate() {
        //     let rgb = if i == self.active_game_index {
        //         RGB::new(255, 255, 255) // Highlight selected game
        //     } else {
        //         RGB::new(100, 100, 100)
        //     };
        //     // Display game name or some representation
        //     render_board.set(i % GRID_SIZE, i / GRID_SIZE, rgb);
        // }
        //Ok(render_board)
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
    fn test_select_game() {
        let mut menu = Menu::new();
        menu.active_game_index = 0;
        menu.select_game();
        assert!(matches!(
            menu.state,
            MenuState::RunningGame(GameType::ConnectFour(_))
        ));

        menu.active_game_index = 1;
        menu.select_game();
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
        menu.select_game();

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

    // Add more tests for other scenarios as needed
}
