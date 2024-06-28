use crate::animation::Animation;
use crate::game::{
    ButtonState, CommandType, Game, GameCommand, Player as GamePlayer, RenderBoard, RGB,
};
use crate::GameError;

use crate::random::CustomRng;
use core::time::Duration;
use smallvec::SmallVec;

const GRID_SIZE: usize = 12;
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);
const VICTORY_ANIMATION_DURATION: Duration = Duration::from_secs(5);
const VICTORY_ANIMATION_SPEED: Duration = Duration::from_millis(100);
const WALLS_PER_COLOR: usize = 5;
const MAX_WALLS: usize = 50; // 10 colors * 5 walls per color

const DANGER_COLORS: [(u8, u8, u8); 10] = [
    (0, 255, 0),     // Green (least danger)
    (144, 238, 144), // Light Green
    (173, 216, 230), // Light Blue
    (255, 255, 0),   // Yellow
    (255, 165, 0),   // Orange
    (255, 99, 71),   // Tomato
    (255, 0, 0),     // Red
    (255, 0, 255),   // Magenta
    (128, 0, 128),   // Purple
    (139, 0, 0),     // Dark Red (most danger)
];

struct Player {
    col: usize,
    row: usize,
    is_alive: bool,
    color: RGB,
}

impl Player {
    fn new(player: GamePlayer) -> Self {
        Self {
            col: 0,
            row: match player {
                GamePlayer::Player1 => GRID_SIZE / 3,
                GamePlayer::Player2 => 2 * GRID_SIZE / 3,
            },
            is_alive: true,
            color: player_color(player),
        }
    }

    fn move_down(&mut self) {
        if self.row > 0 {
            self.row -= 1;
        }
    }

    fn move_up(&mut self) {
        if self.row < GRID_SIZE - 1 {
            self.row += 1;
        }
    }
}

struct Wall {
    col: usize,
    gap_row: usize,
    gap_size: usize,
}

impl Wall {
    fn new(col: usize, gap_row: usize, gap_size: usize) -> Self {
        Self {
            col,
            gap_row,
            gap_size,
        }
    }
}

enum GameState {
    Playing,
    GameOver,
    Victory(Duration),
}

pub struct WallDodger {
    state: GameState,
    players: SmallVec<[Player; 2]>,
    walls: SmallVec<[Wall; GRID_SIZE]>,
    current_time: Duration,
    wall_gap: usize,
    wall_period: f64,
    last_wall_time: f64,
    rng: CustomRng,
    game_over_animation: Animation,
    victory_animation: Animation,
    walls_passed: usize,
    color_index: usize,
    is_multiplayer: bool,
}

impl WallDodger {
    pub fn new(seed: u64, is_multiplayer: bool) -> Self {
        let mut players = SmallVec::new();
        players.push(Player::new(GamePlayer::Player1));
        if is_multiplayer {
            players.push(Player::new(GamePlayer::Player2));
        }

        Self {
            state: GameState::Playing,
            players,
            walls: SmallVec::new(),
            current_time: Duration::ZERO,
            wall_gap: 8,
            wall_period: 0.18,
            last_wall_time: 0.0,
            rng: CustomRng::seed_from_u64(seed),
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
            victory_animation: Animation::new(VICTORY_ANIMATION_SPEED),
            walls_passed: 0,
            color_index: 0,
            is_multiplayer,
        }
    }

    fn move_walls_left(&mut self) {
        let mut wall_passed = false;
        self.walls.retain_mut(|wall| {
            if wall.col == 0 {
                false
            } else {
                wall.col -= 1;
                if wall.col == 0 {
                    wall_passed = true;
                }
                true
            }
        });
        if wall_passed {
            self.walls_passed += 1;
            self.level_up();
        }
    }

    fn add_wall(&mut self) {
        const GAP_SIZE: usize = 4;
        let gap_row = self.rng.gen_range(0, (GRID_SIZE - GAP_SIZE + 1) as u32) as usize;
        self.walls.push(Wall::new(GRID_SIZE - 1, gap_row, GAP_SIZE));
    }

    fn detect_collisions(&mut self) {
        for player in self.players.iter_mut() {
            if player.is_alive {
                for wall in &self.walls {
                    if wall.col == player.col
                        && !(wall.gap_row <= player.row
                            && player.row < wall.gap_row + wall.gap_size)
                    {
                        player.is_alive = false;
                        break;
                    }
                }
            }
        }

        if !self.players.iter().any(|p| p.is_alive) {
            self.state = GameState::GameOver;
        }
    }

    fn level_up(&mut self) {
        if self.walls_passed % WALLS_PER_COLOR == 0 {
            self.color_index += 1;
            self.wall_period *= 0.9; // Increase speed by 10% every 5 walls
        }
        if self.walls_passed >= MAX_WALLS {
            self.state = GameState::Victory(Duration::ZERO);
        }
    }

    fn reset_game(&mut self) {
        self.state = GameState::Playing;
        self.walls.clear();
        self.players.clear();
        self.players.push(Player::new(GamePlayer::Player1));
        if self.is_multiplayer {
            self.players.push(Player::new(GamePlayer::Player2));
        }
        self.walls_passed = 0;
        self.color_index = 0;
        self.wall_period = 0.18;
    }

    fn wall_color(&self) -> RGB {
        let (r, g, b) = DANGER_COLORS[self.color_index % DANGER_COLORS.len()];
        RGB::new(r, g, b)
    }

    fn blend_colors(color1: RGB, color2: RGB) -> RGB {
        RGB::new(
            (color1.r as u16 + color2.r as u16) as u8 / 2,
            (color1.g as u16 + color2.g as u16) as u8 / 2,
            (color1.b as u16 + color2.b as u16) as u8 / 2,
        )
    }

    fn generate_psychedelic_color(&self, row: usize, col: usize, time: Duration) -> RGB {
        let time_factor = time.as_secs_f32() * 2.0;
        let r = ((libm::sinf((row as f32 * 0.3 + time_factor) * 2.0) + 1.0) * 127.5) as u8;
        let g = ((libm::sinf((col as f32 * 0.3 + time_factor) * 2.0 + 2.094) + 1.0) * 127.5) as u8;
        let b = ((libm::sinf((row as f32 + col as f32) * 0.3 + time_factor * 2.0 + 4.188) + 1.0)
            * 127.5) as u8;
        RGB::new(r, g, b)
    }
}

impl Game for WallDodger {
    fn process_input(&mut self, input_command: GameCommand) -> Result<(), GameError> {
        if let ButtonState::Pressed = input_command.button_state {
            match (&self.state, input_command.command_type) {
                (GameState::Playing, CommandType::Up) => {
                    if let Some(player) = self
                        .players
                        .iter_mut()
                        .find(|p| p.is_alive && p.color == player_color(input_command.player))
                    {
                        player.move_up();
                    }
                }
                (GameState::Playing, CommandType::Down) => {
                    if let Some(player) = self
                        .players
                        .iter_mut()
                        .find(|p| p.is_alive && p.color == player_color(input_command.player))
                    {
                        player.move_down();
                    }
                }
                (GameState::GameOver, CommandType::Select)
                | (GameState::Victory(_), CommandType::Select) => self.reset_game(),
                _ => {}
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.current_time += delta_time;

        match &mut self.state {
            GameState::Playing => {
                let dt = delta_time.as_secs_f64();

                self.detect_collisions();

                self.last_wall_time += dt;
                if self.last_wall_time > self.wall_period {
                    self.move_walls_left();
                    self.last_wall_time = 0.0;
                }

                if self.walls.is_empty()
                    || self
                        .walls
                        .last()
                        .map_or(false, |w| w.col == GRID_SIZE - self.wall_gap)
                {
                    self.add_wall();
                }
            }
            GameState::GameOver => {
                self.game_over_animation.update(self.current_time);
            }
            GameState::Victory(elapsed_time) => {
                *elapsed_time += delta_time;
                self.victory_animation.update(self.current_time);
                if *elapsed_time >= VICTORY_ANIMATION_DURATION {
                    self.reset_game();
                }
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard, GameError> {
        let mut render_board = RenderBoard::new();

        match &self.state {
            GameState::Playing | GameState::GameOver => {
                let wall_color = self.wall_color();
                for wall in &self.walls {
                    for row in (0..wall.gap_row).chain(wall.gap_row + wall.gap_size..GRID_SIZE) {
                        render_board.set(wall.col, row, wall_color);
                    }
                }

                for player in &self.players {
                    if player.is_alive {
                        render_board.set(player.col, player.row, player.color);
                    }
                }

                if self.players.len() == 2 && self.players[0].row == self.players[1].row {
                    let blended_color =
                        Self::blend_colors(self.players[0].color, self.players[1].color);
                    render_board.set(self.players[0].col, self.players[0].row, blended_color);
                }

                if let GameState::GameOver = self.state {
                    if let Some(first_wall) = self.walls.first() {
                        if first_wall.col == 0 {
                            let color = self.game_over_animation.get_color();
                            for row in (0..first_wall.gap_row)
                                .chain(first_wall.gap_row + first_wall.gap_size..GRID_SIZE)
                            {
                                render_board.set(0, row, color);
                            }
                        }
                    }
                }
            }
            GameState::Victory(elapsed_time) => {
                for row in 0..GRID_SIZE {
                    for col in 0..GRID_SIZE {
                        let color = self.generate_psychedelic_color(row, col, *elapsed_time);
                        render_board.set(col, row, color);
                    }
                }
            }
        }

        Ok(render_board)
    }
}

fn player_color(player: GamePlayer) -> RGB {
    match player {
        GamePlayer::Player1 => RGB::new(0, 255, 0), // Green
        GamePlayer::Player2 => RGB::new(0, 0, 255), // Blue
    }
}
