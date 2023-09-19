pub mod connect_four;

use anyhow::Result;
pub const GRID_SIZE: usize = 12;

pub trait Game {
    fn process_input(&mut self) -> Result<()>;
    fn update(&mut self) -> Result<()>;
    fn render(&self) -> Result<()>;
}

pub struct Board<T> {
    pub cells: [[T; 12]; 12],
}

impl<T: Copy> Board<T> {
    pub fn new(initial_value: T) -> Self
    where
        T: Copy,
    {
        Board {
            cells: [[initial_value; 12]; 12],
        }
    }
    pub fn size(&self) -> usize {
        self.cells.len()
    }
    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.cells[self.size() - 1 - y][x] = value;
    }
    pub fn get(&self, x: usize, y: usize) -> T {
        self.cells[self.size() - 1 - y][x]
    }
}

//type GameBoard = Board<Cell>;
//type RenderBoard = Board<RGB>;

pub struct GameBoard {
    pub cells: [[Cell; 12]; 12],
}

impl GameBoard {
    pub fn new() -> Self {
        Self {
            cells: Default::default(),
        }
    }

    pub fn size(&self) -> usize {
        self.cells.len()
    }

    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        self.cells[row][col] = cell;
    }
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[self.size() - 1 - y][x] = cell;
    }
    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[self.size() - 1 - y][x]
    }

    pub fn print(&self) {
        for row in self.cells.iter() {
            for cell in row.iter() {
                match cell {
                    Cell::Empty => {}//print!(". "),
                    Cell::PlayerX => {}//print!("X "),
                    Cell::PlayerO => {}//print!("O "),
                }
            }
            //println!();
        }
        //println!("");
    }
    /*
        pub fn map_as_rgb(&self) -> Vec<Vec<RGB>> {
            let mut map = vec![vec![RGB::new(0, 0, 0); self.size()]; self.size()];
            for row in 0..self.size() {
                for col in 0..self.size() {
                    match self.get(col, row) {
                        Cell::Empty => map[row][col] = RGB::new(0, 0, 0),
                        Cell::PlayerX => map[row][col] = RGB::new(255, 0, 0),
                        Cell::PlayerO => map[row][col] = RGB::new(0, 0, 255),
                    }
                }
            }
            map
        }

        pub fn print_as_rgb(&self) {
            for row in self.cells.iter() {
                for cell in row.iter() {
                    match cell {
                        Cell::Empty => print!("{:?} ", RGB::new(0, 0, 0)),
                        Cell::PlayerX => print!("{:?} ", RGB::new(255, 0, 0)),
                        Cell::PlayerO => print!("{:?} ", RGB::new(0, 0, 255)),
                    }
                }
                //println!();
            }
            //println!();
        }
    */
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    PlayerX,
    PlayerO,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameCommand {
    Left,
    Right,
    Down,
    Up,
    Quit,
    Select,
}
