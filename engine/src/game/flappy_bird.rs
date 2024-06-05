use crate::game::{ButtonState, CommandType, Game, GameCommand, RenderBoard, Result, RGB};

use core::time::Duration;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use smallvec::SmallVec;
const GRID_SIZE: usize = 12;
const GAME_OVER_ANIMATION_SPEED: Duration = Duration::from_millis(50);
use crate::animation::Animation;

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

        let speed_multiplier = 15.0; //10.0;
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
    current_time: Duration,
    wall_gap: usize,
    wall_period: f64,
    last_wall_time: f64,
    smallrng: SmallRng,
    game_over_animation: Animation,
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
            current_time: Duration::from_millis(0),
            wall_gap: 8,
            wall_period: 0.18,
            last_wall_time: 0.0,
            smallrng: SmallRng::seed_from_u64(55098345123984287), // maybe see if you can get this from
            // system time or something
            game_over_animation: Animation::new(GAME_OVER_ANIMATION_SPEED),
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
                if let ButtonState::Pressed = input_command.button_state {
                    match input_command.command_type {
                        CommandType::Select => {
                            self.state = FlappyBirdState::Playing;
                            self.walls.clear();
                            self.player = Player::new();
                        }
                        _ => return Ok(()),
                    }
                }

                // TODO
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_time: Duration) -> Result<()> {
        self.current_time += delta_time;
        match &self.state {
            FlappyBirdState::Playing => {
                let dt = delta_time.as_secs_f64();
                self.player.update_position(dt);
                if self.detect_collisions() {
                    self.state = FlappyBirdState::GameOver;
                }

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

                self.last_wall_time += dt;
            }
            FlappyBirdState::GameOver => {
                self.game_over_animation.update(self.current_time);
            }
        }
        Ok(())
    }

    // fn render(&self) -> Result<RenderBoard> {
    //     let mut render_board = RenderBoard::new();
    //     match &self.state {
    //         FlappyBirdState::Playing => {
    //             for wall in self.walls.iter() {
    //                 for row in (0..wall.gap_row).chain((wall.gap_row + wall.gap_size)..GRID_SIZE) {
    //                     render_board.set(wall.col, row, RGB::new(255, 0, 0));
    //                 }
    //             }
    //             render_board.set(self.player.col, self.player.row(), RGB::new(0, 255, 0));
    //         }
    //         FlappyBirdState::GameOver => {
    //             render_board.set(self.player.col, self.player.row(), RGB::new(0, 255, 0));
    //
    //             if let Some(first) = self.walls.first() {
    //                 if first.col == 0 {
    //                     for row in
    //                         (0..first.gap_row).chain((first.gap_row + first.gap_size)..GRID_SIZE)
    //                     {
    //                         let s = self.game_over_animation_state.state;
    //                         let f: f64 = s as f64;
    //                         let s = fabs(sin(f * 2.0 * 3.141 / 20.0)) * 10.0 + 10.0;
    //                         let color = RGB::new(s as u8 * 10, s as u8 * 10, s as u8 * 10);
    //                         render_board.set(0, row, color);
    //                     }
    //                 }
    //                 render_board.set(self.player.col, self.player.row(), RGB::new(189, 20, 20));
    //             }
    //         }
    //     }
    //
    //     Ok(render_board)
    // }
    fn render(&self) -> Result<RenderBoard> {
        let mut render_board = RenderBoard::new();
        match &self.state {
            FlappyBirdState::Playing => {
                for wall in self.walls.iter() {
                    for row in (0..wall.gap_row).chain((wall.gap_row + wall.gap_size)..GRID_SIZE) {
                        render_board.set(wall.col, row, RGB::new(255, 0, 0));
                    }
                }
                render_board.set(self.player.col, self.player.row(), RGB::new(0, 255, 0));
            }
            FlappyBirdState::GameOver => {
                render_board.set(self.player.col, self.player.row(), RGB::new(0, 255, 0));
                if let Some(first) = self.walls.first() {
                    if first.col == 0 {
                        let color = self.game_over_animation.get_color();
                        for row in
                            (0..first.gap_row).chain((first.gap_row + first.gap_size)..GRID_SIZE)
                        {
                            render_board.set(0, row, color);
                        }
                    }
                    render_board.set(self.player.col, self.player.row(), RGB::new(189, 20, 20));
                }
            }
        }
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
