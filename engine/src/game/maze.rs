use crate::{
    GameError, Player as GamePlayer,
    animation::Animation,
    game::{ButtonState, CommandType, Game, GameCommand, RGB, RenderBoard},
};

use crate::random::CustomRng;
use core::time::Duration;
use libm::sqrt;
use smallvec::SmallVec;

const VICTORY_ANIMATION_DURATION: Duration = Duration::from_secs(5);
const VICTORY_ANIMATION_SPEED: Duration = Duration::from_millis(100);
const GRID_SIZE: usize = 12;

struct Player {
    position: (usize, usize),
    color: RGB,
}

impl Player {
    fn new(position: (usize, usize), color: RGB) -> Self {
        Self { position, color }
    }
}

pub struct MazeGame {
    board: MazeBoard,
    state: MazeGameState,
    mode: MazeGameMode,
    players: SmallVec<[Player; 2]>,
    exit_pos: (usize, usize),
    victory_animation: Animation,
    current_time: Duration,
    winning_player: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MazeGameState {
    Playing,
    Victory(Duration),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MazeGameMode {
    Normal,
    Multiplayer,
    FlashLight,
    FlashLightMultiplayer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MazeTile {
    Empty,
    Wall,
}

struct MazeBoard {
    tiles: [[MazeTile; GRID_SIZE]; GRID_SIZE],
    seed: u64,
}

impl MazeBoard {
    fn new(seed: u64) -> Self {
        let mut tiles = [[MazeTile::Wall; GRID_SIZE]; GRID_SIZE];
        let mut size = GRID_SIZE;
        if GRID_SIZE.is_multiple_of(2) {
            size += 1;
        }
        let mut rng = CustomRng::seed_from_u64(seed);
        let start_pos = (1, 1);
        tiles[start_pos.1 as usize][start_pos.0 as usize] = MazeTile::Empty;
        let mut directions: [(isize, isize); 4] = [(-2, 0), (2, 0), (0, -2), (0, 2)];
        let mut stack = SmallVec::<[(isize, isize); 128]>::new();

        stack.insert(0, start_pos);

        while !stack.is_empty() {
            let (x, y) = stack.last().unwrap();
            rng.shuffle(&mut directions);

            let mut moved = false;
            for (dx, dy) in directions.iter() {
                let nx = x + dx;
                let ny = y + dy;

                if 0 < nx
                    && nx < size as isize
                    && 0 < ny
                    && ny < size as isize
                    && tiles[ny as usize][nx as usize] == MazeTile::Wall
                {
                    tiles[ny as usize][nx as usize] = MazeTile::Empty;
                    tiles[(y + dy / 2) as usize][(x + dx / 2) as usize] = MazeTile::Empty;
                    stack.push((nx, ny));
                    moved = true;
                    break;
                }
            }
            if !moved {
                stack.pop();
            }
        }
        Self { tiles, seed }
    }

    fn find_furthest_tile(&self, start_pos: (usize, usize)) -> (usize, usize) {
        let mut stack = SmallVec::<[(usize, usize, usize); 128]>::new();
        let mut visited = [[false; GRID_SIZE]; GRID_SIZE];
        let mut max_distance = 0;
        let mut furthest_tile = start_pos;

        stack.push((start_pos.0, start_pos.1, 0));
        visited[start_pos.0][start_pos.1] = true;

        while let Some((x, y, distance)) = stack.pop() {
            if distance > max_distance {
                max_distance = distance;
                furthest_tile = (x, y);
            }

            for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let nx = (x as isize + dx) as usize;
                let ny = (y as isize + dy) as usize;

                if nx < GRID_SIZE
                    && ny < GRID_SIZE
                    && self.tiles[nx][ny] == MazeTile::Empty
                    && !visited[nx][ny]
                {
                    stack.push((nx, ny, distance + 1));
                    visited[nx][ny] = true;
                }
            }
        }
        furthest_tile
    }
}

impl MazeGame {
    pub fn new(seed: u64, mode: MazeGameMode) -> Self {
        let board = MazeBoard::new(seed);
        let start_pos = (1, 1);
        let exit_pos = board.find_furthest_tile(start_pos);

        let mut players = SmallVec::new();
        players.push(Player::new(start_pos, RGB::new(0, 255, 0))); // Player 1: Green

        if let MazeGameMode::Multiplayer | MazeGameMode::FlashLightMultiplayer = mode {
            players.push(Player::new(start_pos, RGB::new(0, 0, 255))); // Player 2: Blue
        }

        Self {
            board,
            state: MazeGameState::Playing,
            mode,
            players,
            exit_pos,
            victory_animation: Animation::new(VICTORY_ANIMATION_SPEED),
            current_time: Duration::from_millis(0),
            winning_player: None,
        }
    }

    fn generate_psychedelic_color(
        &self,
        row: usize,
        col: usize,
        time: Duration,
        winner_color: RGB,
    ) -> RGB {
        let time_factor = time.as_secs_f32() * 2.0;
        let base_r = ((libm::sinf((row as f32 * 0.3 + time_factor) * 2.0) + 1.0) * 127.5) as u8;
        let base_g =
            ((libm::sinf((col as f32 * 0.3 + time_factor) * 2.0 + 2.094) + 1.0) * 127.5) as u8;
        let base_b = ((libm::sinf((row as f32 + col as f32) * 0.3 + time_factor * 2.0 + 4.188)
            + 1.0)
            * 127.5) as u8;

        // Blend the base psychedelic color with the winner's color
        let blend_factor = 0.7; // Adjust this to control how prominent the winner's color is
        let r = (base_r as f32 * (1.0 - blend_factor) + winner_color.r as f32 * blend_factor) as u8;
        let g = (base_g as f32 * (1.0 - blend_factor) + winner_color.g as f32 * blend_factor) as u8;
        let b = (base_b as f32 * (1.0 - blend_factor) + winner_color.b as f32 * blend_factor) as u8;

        RGB::new(r, g, b)
    }

    fn blend_colors(color1: RGB, color2: RGB) -> RGB {
        RGB::new(
            (color1.r as u16 + color2.r as u16) as u8 / 2,
            (color1.g as u16 + color2.g as u16) as u8 / 2,
            (color1.b as u16 + color2.b as u16) as u8 / 2,
        )
    }

    fn reset_game(&mut self) {
        self.board = MazeBoard::new(self.board.seed + 1);
        self.state = MazeGameState::Playing;
        self.winning_player = None;

        let start_pos = (1, 1);
        self.exit_pos = self.board.find_furthest_tile(start_pos);

        for player in &mut self.players {
            player.position = start_pos;
        }
    }
}

impl Game for MazeGame {
    fn process_input(&mut self, input: GameCommand) -> Result<(), GameError> {
        match &mut self.state {
            MazeGameState::Playing => {
                if let ButtonState::Pressed = input.button_state {
                    let (dx, dy) = match input.command_type {
                        CommandType::Left => (-1, 0),
                        CommandType::Right => (1, 0),
                        CommandType::Up => (0, 1),
                        CommandType::Down => (0, -1),
                        _ => (0, 0),
                    };

                    let player_index = match (self.mode, input.player) {
                        (MazeGameMode::Normal | MazeGameMode::FlashLight, _) => 0,
                        (_, GamePlayer::Player1) => 0,
                        (_, GamePlayer::Player2) => 1,
                    };

                    if let Some(player) = self.players.get_mut(player_index) {
                        let (x, y) = player.position;
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;

                        if nx >= 0
                            && nx < GRID_SIZE as isize
                            && ny >= 0
                            && ny < GRID_SIZE as isize
                            && self.board.tiles[nx as usize][ny as usize] != MazeTile::Wall
                        {
                            player.position = (nx as usize, ny as usize);
                        }

                        if player.position == self.exit_pos {
                            self.state = MazeGameState::Victory(Duration::ZERO);
                            self.winning_player = Some(player_index);
                        }
                    }
                }
            }
            MazeGameState::Victory(elapsed_time) => {
                if input.command_type == CommandType::Select
                    && *elapsed_time >= VICTORY_ANIMATION_DURATION
                {
                    self.reset_game();
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), GameError> {
        self.current_time += delta_time;

        match &mut self.state {
            MazeGameState::Playing => {}
            MazeGameState::Victory(elapsed_time) => {
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
            MazeGameState::Playing => {
                match &self.mode {
                    MazeGameMode::Normal | MazeGameMode::Multiplayer => {
                        // Render maze
                        for x in 0..GRID_SIZE {
                            for y in 0..GRID_SIZE {
                                let rgb = match self.board.tiles[x][y] {
                                    MazeTile::Empty => RGB::new(0, 0, 0),
                                    MazeTile::Wall => RGB::new(255, 255, 255),
                                };
                                render_board.set(x, y, rgb);
                            }
                        }
                        // Render exit
                        render_board.set(self.exit_pos.0, self.exit_pos.1, RGB::new(255, 0, 0));
                    }
                    MazeGameMode::FlashLight | MazeGameMode::FlashLightMultiplayer => {
                        let distance = |x: usize, y: usize, player: &Player| {
                            let dx = x as isize - player.position.0 as isize;
                            let dy = y as isize - player.position.1 as isize;
                            sqrt((dx * dx + dy * dy) as f64)
                        };

                        for x in 0..GRID_SIZE {
                            for y in 0..GRID_SIZE {
                                let mut max_intensity = 0;
                                for player in &self.players {
                                    let d = distance(x, y, player);
                                    let intensity = (255.0 * (1.0 - d / 3.0)).max(0.0) as u8;
                                    max_intensity = max_intensity.max(intensity);
                                }

                                let rgb = match self.board.tiles[x][y] {
                                    MazeTile::Empty => RGB::new(0, 0, 0),
                                    MazeTile::Wall => {
                                        RGB::new(max_intensity, max_intensity, max_intensity)
                                    }
                                };
                                render_board.set(x, y, rgb);
                            }
                        }

                        // Render exit
                        let exit_intensity = self
                            .players
                            .iter()
                            .map(|p| distance(self.exit_pos.0, self.exit_pos.1, p))
                            .map(|d| (255.0 * (1.0 - d / 3.0)).max(0.0) as u8)
                            .max()
                            .unwrap_or(0);
                        render_board.set(
                            self.exit_pos.0,
                            self.exit_pos.1,
                            RGB::new(exit_intensity, 0, 0),
                        );
                    }
                }

                // Render players with blending
                if self.players.len() == 2 && self.players[0].position == self.players[1].position {
                    let blended_color =
                        Self::blend_colors(self.players[0].color, self.players[1].color);
                    render_board.set(
                        self.players[0].position.0,
                        self.players[0].position.1,
                        blended_color,
                    );
                } else {
                    for player in &self.players {
                        render_board.set(player.position.0, player.position.1, player.color);
                    }
                }
            }
            MazeGameState::Victory(elapsed_time) => {
                if let Some(winner_index) = self.winning_player {
                    let winner_color = self.players[winner_index].color;
                    for row in 0..GRID_SIZE {
                        for col in 0..GRID_SIZE {
                            let color = self.generate_psychedelic_color(
                                row,
                                col,
                                *elapsed_time,
                                winner_color,
                            );
                            render_board.set(col, row, color);
                        }
                    }
                }

                // Render players with blending
                if self.players.len() == 2 && self.players[0].position == self.players[1].position {
                    let blended_color =
                        Self::blend_colors(self.players[0].color, self.players[1].color);
                    render_board.set(
                        self.players[0].position.0,
                        self.players[0].position.1,
                        blended_color,
                    );
                } else {
                    for player in &self.players {
                        render_board.set(player.position.0, player.position.1, player.color);
                    }
                }
            }
        }
        Ok(render_board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{ButtonState, CommandType, GameCommand, Player};

    #[test]
    fn test_maze_board_creation() {
        let seed = 42;
        let board = MazeBoard::new(seed);
        assert_eq!(board.seed, seed);
        assert_eq!(board.tiles[1][1], MazeTile::Empty);
    }

    #[test]
    fn test_find_furthest_tile() {
        let seed = 42;
        let board = MazeBoard::new(seed);
        let start_pos = (1, 1);
        let furthest_tile = board.find_furthest_tile(start_pos);
        assert!(furthest_tile.0 < GRID_SIZE);
        assert!(furthest_tile.1 < GRID_SIZE);
    }

    #[test]
    fn test_maze_game_creation() {
        let seed = 42;
        let game = MazeGame::new(seed, MazeGameMode::Normal);
        assert_eq!(game.state, MazeGameState::Playing);
        assert_eq!(game.player_pos[0], (1, 1));
        assert_ne!(game.exit_pos, (1, 1));
    }

    #[test]
    fn check_collision() {
        let seed = 42;
        let mut game = MazeGame::new(seed, MazeGameMode::Normal);

        let left_command = GameCommand {
            command_type: CommandType::Left,
            button_state: ButtonState::Pressed,
            player: Player::Player1,
        };

        let up_tile = game.board.tiles[1][2];
        game.process_input(left_command).unwrap();
        if up_tile == MazeTile::Wall {
            assert_eq!(game.player_pos[0], (1, 1));
        } else {
            assert_eq!(game.player_pos[0], (0, 1));
        }
        assert_eq!(game.player_pos[0], (1, 1));
    }

    #[test]
    fn test_process_input_game_over() {
        let seed = 42;
        let mut game = MazeGame::new(seed, MazeGameMode::Normal);

        // Simulate reaching the exit
        game.player_pos[0] = game.exit_pos;
        let right_command = GameCommand {
            command_type: CommandType::Right,
            button_state: ButtonState::Pressed,
            player: Player::Player1,
        };

        game.process_input(right_command).unwrap();
        assert_eq!(game.state, MazeGameState::GameOver);

        // Restart game
        let select_command = GameCommand {
            command_type: CommandType::Select,
            button_state: ButtonState::Pressed,
            player: Player::Player1,
        };

        game.process_input(select_command).unwrap();
        assert_eq!(game.state, MazeGameState::Playing);
        assert_eq!(game.player_pos[0], (1, 1));
    }

    #[test]
    fn test_update_game_over_animation() {
        let seed = 42;
        let mut game = MazeGame::new(seed, MazeGameMode::Normal);

        // Simulate reaching the exit
        game.player_pos[0] = game.exit_pos;
        let right_command = GameCommand {
            command_type: CommandType::Right,
            button_state: ButtonState::Pressed,
            player: Player::Player1,
        };

        game.process_input(right_command).unwrap();
        game.update(Duration::from_millis(100)).unwrap();

        assert!(game.win_animation.get_color().r > 0); // Assuming win_animation changes color over time
    }
}
