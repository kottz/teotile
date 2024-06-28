use embassy_rp::gpio::{AnyPin, Input, Pull};
use embassy_time::{Duration, Instant, Timer};
use heapless::spsc::Queue;

#[derive(Clone, Copy)]
pub enum GamepadEvent {
    DPadUp(usize),
    DPadDown(usize),
    DPadLeft(usize),
    DPadRight(usize),
    South(usize),
    East(usize),
}

pub struct GamepadHandler {
    players: [PlayerInputs; 2],
}

struct Button {
    input: Input<'static>,
    state: ButtonState,
    player_id: usize,
}

#[derive(Clone, Copy)]
struct ButtonState {
    is_pressed: bool,
    last_change: Instant,
}

struct PlayerInputs {
    up: Button,
    down: Button,
    left: Button,
    right: Button,
    south: Button,
    east: Button,
    event_queue: Queue<GamepadEvent, 16>,
}

impl Button {
    fn new(pin: AnyPin, player_id: usize) -> Self {
        Self {
            input: Input::new(pin, Pull::Up),
            state: ButtonState {
                is_pressed: false,
                last_change: Instant::now(),
            },
            player_id,
        }
    }

    fn check(
        &mut self,
        event: fn(usize) -> GamepadEvent,
        event_queue: &mut Queue<GamepadEvent, 16>,
    ) {
        let now = Instant::now();
        let is_pressed = self.input.is_low();

        if is_pressed != self.state.is_pressed
            && now - self.state.last_change > Duration::from_millis(20)
        {
            self.state.is_pressed = is_pressed;
            self.state.last_change = now;

            if is_pressed {
                event_queue.enqueue(event(self.player_id)).ok();
            }
        }
    }
}

impl PlayerInputs {
    fn new(
        up: AnyPin,
        down: AnyPin,
        left: AnyPin,
        right: AnyPin,
        south: AnyPin,
        east: AnyPin,
        player_id: usize,
    ) -> Self {
        Self {
            up: Button::new(up, player_id),
            down: Button::new(down, player_id),
            left: Button::new(left, player_id),
            right: Button::new(right, player_id),
            south: Button::new(south, player_id),
            east: Button::new(east, player_id),
            event_queue: Queue::new(),
        }
    }

    fn check_buttons(&mut self) {
        self.up.check(GamepadEvent::DPadUp, &mut self.event_queue);
        self.down
            .check(GamepadEvent::DPadDown, &mut self.event_queue);
        self.left
            .check(GamepadEvent::DPadLeft, &mut self.event_queue);
        self.right
            .check(GamepadEvent::DPadRight, &mut self.event_queue);
        self.south.check(GamepadEvent::South, &mut self.event_queue);
        self.east.check(GamepadEvent::East, &mut self.event_queue);
    }
}

impl GamepadHandler {
    pub fn new(
        p1_up: AnyPin,
        p1_down: AnyPin,
        p1_left: AnyPin,
        p1_right: AnyPin,
        p1_south: AnyPin,
        p1_east: AnyPin,
        p2_up: AnyPin,
        p2_down: AnyPin,
        p2_left: AnyPin,
        p2_right: AnyPin,
        p2_south: AnyPin,
        p2_east: AnyPin,
    ) -> Self {
        Self {
            players: [
                PlayerInputs::new(p1_up, p1_down, p1_left, p1_right, p1_south, p1_east, 0),
                PlayerInputs::new(p2_up, p2_down, p2_left, p2_right, p2_south, p2_east, 1),
            ],
        }
    }

    pub async fn poll_events(&mut self) {
        for player in &mut self.players {
            player.check_buttons();
        }
        // Add a small delay to avoid excessive polling
        Timer::after(Duration::from_millis(10)).await;
    }

    pub fn get_event(&mut self) -> Option<GamepadEvent> {
        for player in &mut self.players {
            if let Some(event) = player.event_queue.dequeue() {
                return Some(event);
            }
        }
        None
    }
}
