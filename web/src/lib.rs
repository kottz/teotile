use teotile::GRID_SIZE;
use teotile::{ButtonState, CommandType, GameCommand, GameEngine, Player, RGB};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub struct GameWrapper {
    engine: GameEngine,
}

#[wasm_bindgen]
impl GameWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console::log_1(&"Creating new game".into());
        Self {
            engine: GameEngine::default(),
        }
    }

    pub fn process_input(&mut self, command_type: u8, button_state: u8, player: u8) {
        let command_type = match command_type {
            0 => CommandType::Up,
            1 => CommandType::Down,
            2 => CommandType::Left,
            3 => CommandType::Right,
            4 => CommandType::Select,
            5 => CommandType::Quit,
            _ => return,
        };

        let button_state = match button_state {
            0 => ButtonState::Pressed,
            1 => ButtonState::Released,
            _ => return,
        };

        let player = match player {
            0 => Player::Player1,
            1 => Player::Player2,
            _ => return,
        };

        let command = GameCommand::new(command_type, button_state, player);
        let _ = self.engine.process_input(command);
    }

    pub fn update(&mut self, delta: f64) {
        let _ = self
            .engine
            .update(std::time::Duration::from_secs_f64(delta));
    }

    pub fn render(&self) -> Vec<u8> {
        let render_board = self.engine.render().unwrap();
        let mut result = Vec::with_capacity(GRID_SIZE * GRID_SIZE * 3);
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                let color: RGB = render_board.get(i, j);
                result.push(color.r);
                result.push(color.g);
                result.push(color.b);
            }
        }
        result
    }
}
