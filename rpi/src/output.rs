use anyhow::Result;
use std::any::Any;
use teotile::RenderBoard;

pub trait Output: Any {
    fn render(&mut self, render_board: &RenderBoard) -> Result<()>;
    fn as_any_mut(&mut self) -> &mut dyn Any;
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
