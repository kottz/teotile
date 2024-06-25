use gilrs::{Button, Event, EventType, Gilrs};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub enum GamepadEvent {
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    South,
    East,
}

pub struct GamepadHandler {
    event_receiver: Receiver<GamepadEvent>,
}

impl GamepadHandler {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        thread::spawn(move || {
            run_gamepad_loop(sender);
        });
        GamepadHandler {
            event_receiver: receiver,
        }
    }

    pub fn poll_event(&self) -> Option<GamepadEvent> {
        self.event_receiver.try_recv().ok()
    }
}

fn run_gamepad_loop(sender: Sender<GamepadEvent>) {
    let mut gilrs = Gilrs::new().unwrap();

    loop {
        while let Some(Event { event, .. }) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(button, _) => match button {
                    Button::DPadUp => sender.send(GamepadEvent::DPadUp).unwrap(),
                    Button::DPadDown => sender.send(GamepadEvent::DPadDown).unwrap(),
                    Button::DPadLeft => sender.send(GamepadEvent::DPadLeft).unwrap(),
                    Button::DPadRight => sender.send(GamepadEvent::DPadRight).unwrap(),
                    Button::South => sender.send(GamepadEvent::South).unwrap(),
                    Button::East => sender.send(GamepadEvent::East).unwrap(),
                    _ => continue,
                },
                _ => continue,
            }
        }
    }
}
