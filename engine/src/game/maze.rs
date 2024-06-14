use crate::game::{ButtonState, CommandType, Game, GameCommand, RenderBoard, Result, RGB};

use core::time::Duration;
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};
use smallvec::SmallVec;
const GRID_SIZE: usize = 11;
pub struct MazeGame {
    board: MazeBoard,
    state: MazeGameState,
    player_pos: (usize, usize),
    exit_pos: (usize, usize),
    current_time: Duration,
}

enum MazeGameState {
    Playing,
    GameOver,
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum MazeTile {
    Empty,
    Wall,
    Player,
    Exit,
}

struct MazeBoard {
    tiles: [[MazeTile; GRID_SIZE]; GRID_SIZE],
    seed: u64,
}

impl MazeBoard {
    fn new(seed: u64) -> Self {
        let tiles = [[MazeTile::Wall; GRID_SIZE]; GRID_SIZE];
        // TODO here create a maze for this board
        let mut board = Self { tiles, seed };
        board.generate_maze();
        board
    }
    // fn generate_maze(&mut self) {
    //     // ...
    //     // added for debugging
    //     self.tiles = [[MazeTile::Wall; GRID_SIZE]; GRID_SIZE];
    //     let mut rng = SmallRng::seed_from_u64(self.seed);
    //     let start_pos = (1, 1);
    //     self.tiles[start_pos.1 as usize][start_pos.0 as usize] = MazeTile::Empty;
    //
    //     let mut stack = SmallVec::<[(isize, isize); 128]>::new();
    //     for (dx, dy) in &directions {
    //         let nx = start_pos.0 + dx;
    //         let ny = start_pos.1 + dy;
    //         if 0 < nx && nx < GRID_SIZE as isize && 0 < ny && ny < GRID_SIZE as isize {
    //             stack.push((nx, ny));
    //         }
    //     }
    //
    //     while !stack.is_empty() {
    //         let (x, y) = stack.pop().unwrap();
    //         if self.tiles[y as usize][x as usize] == MazeTile::Wall {
    //             self.tiles[y as usize][x as usize] = MazeTile::Empty;
    //
    //             let (px, py) = ((x + start_pos.0) / 2, (y + start_pos.1) / 2);
    //             self.tiles[py as usize][px as usize] = MazeTile::Empty;
    //
    //             directions.shuffle(&mut rng);
    //             for (dx, dy) in &directions {
    //                 let nx = x + dx;
    //                 let ny = y + dy;
    //                 if 0 < nx && nx < GRID_SIZE as isize && 0 < ny && ny < GRID_SIZE as isize {
    //                     stack.push((nx, ny));
    //                 }
    //             }
    //         }
    //     }
    // }
    fn generate_maze(&mut self) {
        // added for debugging
        self.tiles = [[MazeTile::Wall; GRID_SIZE]; GRID_SIZE];
        let mut rng = SmallRng::seed_from_u64(self.seed);
        let start_pos  = (1, 1);
        self.tiles[start_pos.1 as usize][start_pos.0 as usize] = MazeTile::Empty;
        let mut directions: [(isize, isize); 4] = [(-2, 0), (2, 0), (0, -2), (0, 2)];
        let mut stack = SmallVec::<[(isize, isize); 128]>::new();

        stack.insert(0, start_pos);

        while stack.len() > 0 {
            let (x, y) = stack.last().unwrap();
            directions.shuffle(&mut rng);

            let mut moved = false;
            for (dx, dy) in directions.iter() {
                let nx = x + dx;
                let ny = y + dy;

                if 0 < nx
                    && nx < GRID_SIZE as isize
                    && 0 < ny
                    && ny < GRID_SIZE as isize
                    && self.tiles[ny as usize][nx as usize] == MazeTile::Wall
                {
                    self.tiles[ny as usize][nx as usize] = MazeTile::Empty;
                    self.tiles[(y + dy / 2) as usize][(x + dx / 2) as usize] = MazeTile::Empty;
                    stack.push((nx, ny));
                    moved = true;
                    break;
                }
            }
            if !moved {
                stack.pop();
            }
        }
    }
}

impl MazeGame {
    pub fn new(seed: u64) -> Self {
        let board = MazeBoard::new(seed);
        let player_pos = (1, 1);
        let exit_pos = (GRID_SIZE - 2, GRID_SIZE - 2);
        Self {
            board,
            state: MazeGameState::Playing,
            player_pos,
            exit_pos,
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
                let (x, y) = self.player_pos;
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0
                    && nx < GRID_SIZE as isize
                    && ny >= 0
                    && ny < GRID_SIZE as isize
                    && self.board.tiles[nx as usize][ny as usize] != MazeTile::Wall
                {
                    self.player_pos = (nx as usize, ny as usize);
                }
                if self.player_pos == self.exit_pos {
                    self.state = MazeGameState::GameOver;
                }
            }
            MazeGameState::GameOver => {
                if input.command_type == CommandType::Select {
                    self.board = MazeBoard::new(self.board.seed);
                    self.player_pos = (1, 1);
                    // TODO set new exit position after end
                    self.exit_pos = (GRID_SIZE - 2, GRID_SIZE - 2);
                    self.state = MazeGameState::Playing;
                }
            }
        }
        Ok(())
    }
    fn update(&mut self, delta_time: core::time::Duration) -> Result<()> {
        self.current_time += delta_time;
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
        let mut render_board = RenderBoard::new();
        match &self.state {
            MazeGameState::Playing => {
                for x in 0..GRID_SIZE {
                    for y in 0..GRID_SIZE {
                        let rgb = match self.board.tiles[x][y] {
                            MazeTile::Empty => RGB::new(0, 0, 0),
                            MazeTile::Wall => RGB::new(255, 255, 255),
                            MazeTile::Player => RGB::new(0, 255, 0),
                            MazeTile::Exit => RGB::new(255, 0, 0),
                        };
                        render_board.set(x, y, rgb);
                    }
                }
                render_board.set(self.player_pos.0, self.player_pos.1, RGB::new(0, 255, 0));
                render_board.set(self.exit_pos.0, self.exit_pos.1, RGB::new(255, 0, 0));
            }
            MazeGameState::GameOver => {
                // TODO render game over screen
            }
        }
        Ok(render_board)
    }
}
