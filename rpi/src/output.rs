use anyhow::Result;
use teotile::RenderBoard;

pub trait Output {
    fn render(&mut self, render_board: &RenderBoard) -> Result<()>;
}

pub struct DebugOutput;

impl Output for DebugOutput {
    fn render(&mut self, render_board: &RenderBoard) -> Result<()> {
        for row in 0..12 {
            for col in 0..12 {
                let color = render_board.get(col, 11-row);
                print!("({},{},{}) ", color.r, color.g, color.b);
            }
            println!();
        }
        println!();
        Ok(())
    }
}
