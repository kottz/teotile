// struct for the board

// 2 need a way to generate walls and gap in between
//
// 3 need a way to stick gravity on the player
// 4 need a way to make the player jump
// 5 need a way to detect collision

// data strcture for the game walls
// need a y and a length for the gap
// need a function to move every wall to the left
//
//
// 6 we also need some way to convert from continous space to discrete space
// and not make it seem like we are colliding on the ui even if we are not or vice versa

use crate::game::{ButtonState, CommandType, Game, GameCommand, RenderBoard, Result, RGB};

use smallvec::SmallVec;
use core::time::Duration;
use rand::{rngs::SmallRng, Rng, SeedableRng};
const GRID_SIZE: usize = 12;
//pub type FlappyBirdBoard = Board<RGB, 12, 12>;

struct Player {
    col: usize,
    pos: f64,
    r0: f64,
    start_velocity: f64,
    acceleration: f64,
    time: f64,
}

impl Player {
    fn new() -> Self {
        Self {
            col: 0,
            pos: 0.0,
            r0: 0.0,             //start position
            start_velocity: 0.0, // This is effectively the jump height
            acceleration: -0.2,  // this is the gravity
            time: 0.0,
        }
    }

    fn jump(&mut self) {
        self.start_velocity = 0.8; // This is effectively the jump height
                                   //self.start_velocity = 1.0; // This is effectively the jump height
        self.time = 0.0; //resetting the time will make the jump graph start from the beginning,
                         //now with r0 being the current position
        self.r0 = self.pos;
    }

    // https://www.desmos.com/calculator/uaaq7duopk
    // this wont work yet since we need to set the start_velocity to 0 somehow
    // I will implement the rest of the game first so that we can see
    //
    //
    // we also should not reassign every time.
    // The formula only gives us the graph, calculate once and then find
    // the spot with the t variable
    //
    // maybe calculate the formula in the jump function
    // and then just update the position in the update function

    // r = r0 + v0*t + 1/2*a*t^2
    // should probably constantly be calling this function
    // on the update ticks
    // and on a jump we can just set the acceleration or velocity
    // the other way temporarily
    fn update_position(&mut self, delta_time: f64) {
        let new_pos = self.r0
            + self.start_velocity * self.time
            + 0.5 * self.acceleration * self.time * self.time;
        if new_pos < 0.0 {
            self.pos = 0.0;
        } else {
            self.pos = new_pos;
        }
        if self.pos > (GRID_SIZE - 1) as f64 {
            self.start_velocity = 0.0;
            self.pos = (GRID_SIZE - 1) as f64;
        }

        let speed_multiplier = 10.0;
        self.time += speed_multiplier * delta_time;
    }

    fn row(&self) -> usize {
        libm::round(self.pos) as usize
    }
}

pub struct FlappyBird {
    state: FlappyBirdState,
    player: Player,
    walls: SmallVec<[Wall; GRID_SIZE]>,
    wall_gap: usize,
    wall_period: f64,
    last_wall_time: f64,
    smallrng: SmallRng,
}

enum FlappyBirdState {
    Playing,
    GameOver,
}

impl FlappyBird {
    pub fn new() -> Self {
        Self {
            state: FlappyBirdState::Playing,
            player: Player::new(),
            walls: SmallVec::<[Wall; GRID_SIZE]>::new(),
            wall_gap: 8,
            wall_period: 0.3,
            last_wall_time: 0.0,
            smallrng: SmallRng::seed_from_u64(55098345123984287), // maybe see if you can get this from
                                                                  // system time or something
        }
    }

    fn move_walls_left(&mut self) {
        self.walls = self
            .walls
            .drain(..)
            .filter_map(|mut wall| {
                if wall.col == 0 {
                    None
                } else {
                    wall.col -= 1;
                    Some(wall)
                }
            })
            .collect();
    }

    fn add_wall(&mut self) {
        let gap_size = 4;
        let gap_row = self.smallrng.gen_range(0..=GRID_SIZE - gap_size); //-self.wall_gap);
        self.walls.push(Wall::new(10, gap_row, gap_size));
    }

    // If we add multiplayer we might wanna check more than one wall
    // the loop is not necessary otherwise
    fn detect_collisions(&self) -> bool {
        for wall in self.walls.iter() {
            if wall.col == self.player.col
                && !(wall.gap_row <= self.player.row()
                    && self.player.row() < wall.gap_row + wall.gap_size)
            {
                return true;
            }
        }
        false
    }
}

impl Game for FlappyBird {
    fn process_input(&mut self, input_command: GameCommand) -> Result<()> {
        match &self.state {
            FlappyBirdState::Playing => {
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Up | CommandType::Select => {
                            self.player.jump();
                        }
                        _ => {}
                    }
                }
            }
            FlappyBirdState::GameOver => {
                // TODO
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        match &self.state {
            FlappyBirdState::Playing => {
                // this should convert the duration to some sort of float I can send to the player
                let delta_time = delta_time.as_secs_f64();
                self.player.update_position(delta_time);
                if self.detect_collisions() {
                    self.state = FlappyBirdState::GameOver;
                }

                self.last_wall_time += delta_time;

                // if self.last_wall_time > wall_gap as f64 / wall_speed {
                //     //self.move_walls_left();
                //     self.add_wall();
                //     self.last_wall_time = 0.0;
                // }

                // tänk på last wall time som vanlig timer
                if self.last_wall_time > self.wall_period {
                    self.move_walls_left();
                    self.last_wall_time = 0.0;
                }

                if let Some(last) = self.walls.last() {
                    if last.col == GRID_SIZE - self.wall_gap {
                        self.add_wall();
                    }
                }

                if self.walls.is_empty() {
                    self.add_wall();
                }
            }
            FlappyBirdState::GameOver => {
                //TODO
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<RenderBoard> {
        let mut render_board = RenderBoard::new();
        for wall in self.walls.iter() {
            for row in (0..wall.gap_row).chain((wall.gap_row + wall.gap_size)..GRID_SIZE) {
                //render_board[wall.col][row] = RGB::new(255, 0, 0);
                render_board.set(wall.col, row, RGB::new(255, 0, 0));
            }
        }
        render_board.set(self.player.col, self.player.row(), RGB::new(0, 255, 0));

        Ok(render_board)
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
