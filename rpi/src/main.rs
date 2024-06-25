use anyhow::{Context, Result};
use std::time::{Duration, Instant};
use teotile::{ButtonState, CommandType, GameCommand, GameEngine, Player};
mod gamepad;
use gamepad::{GamepadEvent, GamepadHandler};
mod led_strip;
use led_strip::LedStrip;
mod output;
use output::{DebugOutput, Output};

const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);
const LED_COUNT: i32 = 144; // 12x12 grid
const LED_PIN: i32 = 10; // GPIO pin number

// Set this to true to use LED strip output, false to use debug output
const USE_LED_STRIP: bool = true;

fn main() -> Result<()> {
    let mut engine = GameEngine::default();
    let gamepad = GamepadHandler::new();
    let mut output: Box<dyn Output> = if USE_LED_STRIP {
        Box::new(LedStrip::new(LED_PIN, LED_COUNT).context("Failed to initialize LED strip")?)
    } else {
        Box::new(DebugOutput)
    };
    let mut prev_instant = Instant::now();

    println!("Game started. Press Ctrl+C to exit.");

    loop {
        let loop_start = Instant::now();

        // Handle at most one gamepad event per frame
        if let Some(event) = gamepad.poll_event() {
            let command = match event {
                GamepadEvent::DPadUp => {
                    GameCommand::new(CommandType::Up, ButtonState::Pressed, Player::Player1)
                }
                GamepadEvent::DPadDown => {
                    GameCommand::new(CommandType::Down, ButtonState::Pressed, Player::Player1)
                }
                GamepadEvent::DPadLeft => {
                    GameCommand::new(CommandType::Left, ButtonState::Pressed, Player::Player1)
                }
                GamepadEvent::DPadRight => {
                    GameCommand::new(CommandType::Right, ButtonState::Pressed, Player::Player1)
                }
                GamepadEvent::South => {
                    GameCommand::new(CommandType::Select, ButtonState::Pressed, Player::Player1)
                }
                GamepadEvent::East => {
                    GameCommand::new(CommandType::Quit, ButtonState::Pressed, Player::Player1)
                }
            };
            engine
                .process_input(command)
                .context("Failed to process input")?;
        }

        // Calculate delta time
        let current_instant = Instant::now();
        let delta = current_instant - prev_instant;
        prev_instant = current_instant;

        // Update game state
        engine
            .update(delta)
            .context("Failed to update game state")?;

        // Render to output
        let render_board = engine.render().context("Failed to render game state")?;
        output
            .render(&render_board)
            .context("Failed to render to output")?;

        // Calculate how long to sleep to maintain target FPS
        let elapsed = loop_start.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
    }
}
