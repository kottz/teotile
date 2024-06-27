#[derive(Debug)]
pub enum GameError {
    InvalidMove,
    OutOfBounds,
    GameOver,
    // Add more error variants as needed
}

impl core::fmt::Display for GameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GameError::InvalidMove => write!(f, "Invalid move"),
            GameError::OutOfBounds => write!(f, "Out of bounds"),
            GameError::GameOver => write!(f, "Game is over"),
            // Add more error messages as needed
        }
    }
}
