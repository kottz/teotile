use anyhow::{Context, Result};
use clap::Parser;
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Enable debug output
    #[clap(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
            r.store(false, Ordering::SeqCst);
        }
    });

    let mut engine = GameEngine::default();
    let gamepad = GamepadHandler::new();
    let mut output: Box<dyn Output> = if args.debug {
        Box::new(DebugOutput)
    } else {
        Box::new(LedStrip::new(LED_PIN, LED_COUNT).context("Failed to initialize LED strip")?)
    };
    let mut prev_instant = Instant::now();

    println!("Game started. Press Ctrl+C or use 'systemctl stop' to exit.");
    if args.debug {
        println!("Running in debug mode.");
    }

    while running.load(Ordering::Relaxed) {
        let loop_start = Instant::now();

        // Handle gamepad events
        while let Some(event) = gamepad.poll_event() {
            match event {
                GamepadEvent::Connected(id, name) => {
                    println!("Gamepad {} connected: {}", id, name);
                }
                GamepadEvent::Disconnected(id) => {
                    println!("Gamepad {} disconnected", id);
                }
                _ => {
                    if let Some(command) = gamepad_event_to_command(event) {
                        engine
                            .process_input(command)
                            .context("Failed to process input")?;
                    }
                }
            }
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

    // Cleanup
    cleanup(output).context("Failed to cleanup output")?;

    println!("Game stopped. Output cleaned up.");
    Ok(())
}

fn cleanup(mut output: Box<dyn Output>) -> Result<()> {
    if let Some(led_strip) = output.as_any_mut().downcast_mut::<LedStrip>() {
        led_strip.cleanup().context("Failed to cleanup LED strip")?;
    }
    Ok(())
}

fn gamepad_event_to_command(event: GamepadEvent) -> Option<GameCommand> {
    match event {
        GamepadEvent::DPadUp(id) => Some(GameCommand::new(
            CommandType::Up,
            ButtonState::Pressed,
            player_from_id(id),
        )),
        GamepadEvent::DPadDown(id) => Some(GameCommand::new(
            CommandType::Down,
            ButtonState::Pressed,
            player_from_id(id),
        )),
        GamepadEvent::DPadLeft(id) => Some(GameCommand::new(
            CommandType::Left,
            ButtonState::Pressed,
            player_from_id(id),
        )),
        GamepadEvent::DPadRight(id) => Some(GameCommand::new(
            CommandType::Right,
            ButtonState::Pressed,
            player_from_id(id),
        )),
        GamepadEvent::South(id) => Some(GameCommand::new(
            CommandType::Select,
            ButtonState::Pressed,
            player_from_id(id),
        )),
        GamepadEvent::East(id) => Some(GameCommand::new(
            CommandType::Quit,
            ButtonState::Pressed,
            player_from_id(id),
        )),
        GamepadEvent::Connected(_, _) | GamepadEvent::Disconnected(_) => None,
    }
}

fn player_from_id(id: usize) -> Player {
    match id {
        0 => Player::Player1,
        1 => Player::Player2,
        _ => Player::Player1, // Default to Player1 for any additional controllers
    }
}
