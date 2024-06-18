use crate::{
    animation::Animation,
    game::{CommandType, Game, GameCommand, RenderBoard, Result, RGB},
    Player,
};

use core::time::Duration;
use libm::sqrt;
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use smallvec::SmallVec;

const WIN_ANIMATION_SPEED: Duration = Duration::from_millis(50);
const GRID_SIZE: usize = 12;

pub struct MazeGame {
    board: MazeBoard,
    state: MazeGameState,
    mode: MazeGameMode,
    player_pos: [(usize, usize); 2],
    exit_pos: (usize, usize),
    win_animation: Animation,
    current_time: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MazeGameState {
    Playing,
    GameOver,
}

pub enum MazeGameMode {
    Normal,
    FlashLight,
    MultiplayerFlashLight,
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
        if GRID_SIZE % 2 == 0 {
            size += 1;
        }
        let mut rng = SmallRng::seed_from_u64(seed);
        let start_pos = (1, 1);
        tiles[start_pos.1 as usize][start_pos.0 as usize] = MazeTile::Empty;
        let mut directions: [(isize, isize); 4] = [(-2, 0), (2, 0), (0, -2), (0, 2)];
        let mut stack = SmallVec::<[(isize, isize); 128]>::new();

        stack.insert(0, start_pos);

        while !stack.is_empty() {
            let (x, y) = stack.last().unwrap();
            directions.shuffle(&mut rng);

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
        let player_pos = [(1, 1); 2];
        let exit_pos = board.find_furthest_tile(player_pos[0]);
        Self {
            board,
            state: MazeGameState::Playing,
            mode,
            player_pos,
            exit_pos,
            win_animation: Animation::new(WIN_ANIMATION_SPEED),
            current_time: Duration::from_millis(0),
        }
    }
}

impl Game for MazeGame {
    fn process_input(&mut self, input: GameCommand) -> Result<()> {
        match &self.state {
            MazeGameState::Playing => {
                let (dx, dy) = match input.command_type {
                    CommandType::Left => (-1, 0),
                    CommandType::Right => (1, 0),
                    CommandType::Up => (0, 1),
                    CommandType::Down => (0, -1),
                    _ => (0, 0),
                };

                let player_index = if let MazeGameMode::MultiplayerFlashLight = self.mode {
                    match input.player {
                        Player::Player1 => 0,
                        Player::Player2 => 1,
                    }
                } else {
                    0
                };

                let (x, y) = self.player_pos[player_index];
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0
                    && nx < GRID_SIZE as isize
                    && ny >= 0
                    && ny < GRID_SIZE as isize
                    && self.board.tiles[nx as usize][ny as usize] != MazeTile::Wall
                {
                    self.player_pos[player_index] = (nx as usize, ny as usize);
                }

                if self.player_pos[player_index] == self.exit_pos {
                    self.state = MazeGameState::GameOver;
                }
            }
            MazeGameState::GameOver => {
                if input.command_type == CommandType::Select {
                    self.board = MazeBoard::new(self.board.seed + 1);
                    self.player_pos = [(1, 1); 2];
                    self.exit_pos = self.board.find_furthest_tile(self.player_pos[0]);
                    self.state = MazeGameState::Playing;
                }
            }
        }
        Ok(())
    }
    fn update(&mut self, delta_time: core::time::Duration) -> Result<()> {
        self.current_time += delta_time;

        match &self.state {
            MazeGameState::Playing => {}
            MazeGameState::GameOver => {
                self.win_animation.update(self.current_time);
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
        let mut render_board = RenderBoard::new();
        match &self.state {
            MazeGameState::Playing => match &self.mode {
                MazeGameMode::Normal => {
                    for x in 0..GRID_SIZE {
                        for y in 0..GRID_SIZE {
                            let rgb = match self.board.tiles[x][y] {
                                MazeTile::Empty => RGB::new(0, 0, 0),
                                MazeTile::Wall => RGB::new(255, 255, 255),
                            };
                            render_board.set(x, y, rgb);
                        }
                    }
                    render_board.set(self.exit_pos.0, self.exit_pos.1, RGB::new(255, 0, 0));
                    render_board.set(
                        self.player_pos[0].0,
                        self.player_pos[0].1,
                        RGB::new(0, 255, 0),
                    );
                }
                MazeGameMode::FlashLight => {
                    let distance = |x: usize, y: usize| {
                        let dx = x as isize - self.player_pos[0].0 as isize;
                        let dy = y as isize - self.player_pos[0].1 as isize;
                        sqrt((dx * dx + dy * dy) as f64)
                    };
                    for x in 0..GRID_SIZE {
                        for y in 0..GRID_SIZE {
                            let d = distance(x, y);
                            let wall_intensity = (255.0 * (1.0 - d / 3.0)) as u8;
                            let wall_color =
                                RGB::new(wall_intensity, wall_intensity, wall_intensity);
                            let rgb = match self.board.tiles[x][y] {
                                MazeTile::Empty => RGB::new(0, 0, 0),
                                MazeTile::Wall => wall_color,
                            };
                            render_board.set(x, y, rgb);
                        }
                    }
                    let exit_distance = distance(self.exit_pos.0, self.exit_pos.1);
                    let exit_wall_intensity = (255.0 * (1.0 - exit_distance / 3.0)) as u8;
                    let exit_wall_color = RGB::new(exit_wall_intensity, 0, 0);
                    render_board.set(self.exit_pos.0, self.exit_pos.1, exit_wall_color);
                    render_board.set(
                        self.player_pos[0].0,
                        self.player_pos[0].1,
                        RGB::new(0, 255, 0),
                    );
                }
                MazeGameMode::MultiplayerFlashLight => {
                    let distance = |x: usize, y: usize, player_index: usize| {
                        let dx = x as isize - self.player_pos[player_index].0 as isize;
                        let dy = y as isize - self.player_pos[player_index].1 as isize;
                        sqrt((dx * dx + dy * dy) as f64)
                    };
                    for x in 0..GRID_SIZE {
                        for y in 0..GRID_SIZE {
                            let mut max_wall_intensity = 0;

                            for player_index in 0..2 {
                                let d = distance(x, y, player_index);
                                let wall_intensity = (255.0 * (1.0 - d / 3.0)) as u8;

                                if wall_intensity > max_wall_intensity {
                                    max_wall_intensity = wall_intensity;
                                }
                            }

                            let wall_color = RGB::new(
                                max_wall_intensity,
                                max_wall_intensity,
                                max_wall_intensity,
                            );
                            let rgb = match self.board.tiles[x][y] {
                                MazeTile::Empty => RGB::new(0, 0, 0),
                                MazeTile::Wall => wall_color,
                            };

                            render_board.set(x, y, rgb);
                        }
                    }
                    let min_exit_distance = |x: usize, y: usize| {
                        let mut min_d = distance(x, y, 0);
                        for player_index in 0..2 {
                            let d = distance(x, y, player_index);
                            if d < min_d {
                                min_d = d;
                            }
                        }
                        min_d
                    };
                    let exit_distance = min_exit_distance(self.exit_pos.0, self.exit_pos.1);
                    let exit_wall_intensity = (255.0 * (1.0 - exit_distance / 3.0)) as u8;
                    let exit_wall_color = RGB::new(exit_wall_intensity, 0, 0);
                    render_board.set(self.exit_pos.0, self.exit_pos.1, exit_wall_color);
                    render_board.set(
                        self.player_pos[0].0,
                        self.player_pos[0].1,
                        RGB::new(0, 255, 0),
                    );
                    render_board.set(
                        self.player_pos[1].0,
                        self.player_pos[1].1,
                        RGB::new(0, 0, 255),
                    );
                    if self.player_pos[0] == self.player_pos[1] {
                        render_board.set(
                            self.player_pos[0].0,
                            self.player_pos[0].1,
                            RGB::new(0, 128, 128),
                        );
                    }
                }
            },
            MazeGameState::GameOver => {
                let color = self.win_animation.get_color();
                for x in 0..GRID_SIZE {
                    for y in 0..GRID_SIZE {
                        let rgb = match self.board.tiles[x][y] {
                            MazeTile::Empty => RGB::new(0, 0, 0),
                            MazeTile::Wall => color,
                        };
                        render_board.set(x, y, rgb);
                    }
                }
                render_board.set(
                    self.player_pos[0].0,
                    self.player_pos[0].1,
                    RGB::new(0, 255, 0),
                );
                if let MazeGameMode::MultiplayerFlashLight = self.mode {
                    render_board.set(
                        self.player_pos[1].0,
                        self.player_pos[1].1,
                        RGB::new(0, 0, 255),
                    );
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
