use gilrs::{Button, Event, EventType, Gilrs};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub enum GamepadEvent {
    Connected(usize, String),
    Disconnected(usize),
    DPadUp(usize),
    DPadDown(usize),
    DPadLeft(usize),
    DPadRight(usize),
    South(usize),
    East(usize),
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
    let mut active_gamepads = HashMap::new();

    for (id, gamepad) in gilrs.gamepads() {
        let gamepad_id = id.into();
        let name = gamepad.name().to_string();
        active_gamepads.insert(gamepad_id, name.clone());
        sender
            .send(GamepadEvent::Connected(gamepad_id, name))
            .unwrap();
    }

    loop {
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            let gamepad_id: usize = id.into();

            match event {
                EventType::Connected => {
                    let gamepad = gilrs.gamepad(id);
                    let name = gamepad.name().to_string();
                    active_gamepads.insert(gamepad_id, name.clone());
                    sender
                        .send(GamepadEvent::Connected(gamepad_id, name))
                        .unwrap();
                }
                EventType::Disconnected => {
                    active_gamepads.remove(&gamepad_id);
                    sender.send(GamepadEvent::Disconnected(gamepad_id)).unwrap();
                }
                EventType::ButtonPressed(button, _) => {
                    if active_gamepads.contains_key(&gamepad_id) {
                        match button {
                            Button::DPadUp => {
                                sender.send(GamepadEvent::DPadUp(gamepad_id)).unwrap()
                            }
                            Button::DPadDown => {
                                sender.send(GamepadEvent::DPadDown(gamepad_id)).unwrap()
                            }
                            Button::DPadLeft => {
                                sender.send(GamepadEvent::DPadLeft(gamepad_id)).unwrap()
                            }
                            Button::DPadRight => {
                                sender.send(GamepadEvent::DPadRight(gamepad_id)).unwrap()
                            }
                            Button::South => sender.send(GamepadEvent::South(gamepad_id)).unwrap(),
                            Button::East => sender.send(GamepadEvent::East(gamepad_id)).unwrap(),
                            _ => continue,
                        }
                    }
                }
                _ => continue,
            }
        }
    }
}
