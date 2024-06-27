use embassy_rp::gpio::{Input, Pull};
use embassy_time::{Duration, Timer};

pub enum GamepadEvent {
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    South,
    East,
}

pub struct GamepadHandler {
    up: Input<'static>,
    down: Input<'static>,
    left: Input<'static>,
    right: Input<'static>,
    south: Input<'static>,
    east: Input<'static>,
}

impl GamepadHandler {
    pub fn new(
        up: Input<'static>,
        down: Input<'static>,
        left: Input<'static>,
        right: Input<'static>,
        south: Input<'static>,
        east: Input<'static>,
    ) -> Self {
        Self {
            up,
            down,
            left,
            right,
            south,
            east,
        }
    }

    pub async fn poll_event(&self) -> Option<GamepadEvent> {
        if self.up.is_low() {
            Some(GamepadEvent::DPadUp)
        } else if self.down.is_low() {
            Some(GamepadEvent::DPadDown)
        } else if self.left.is_low() {
            Some(GamepadEvent::DPadLeft)
        } else if self.right.is_low() {
            Some(GamepadEvent::DPadRight)
        } else if self.south.is_low() {
            Some(GamepadEvent::South)
        } else if self.east.is_low() {
            Some(GamepadEvent::East)
        } else {
            Timer::after(Duration::from_millis(10)).await;
            None
        }
    }
}
