use gilrs::{Button, Event, Gilrs, EventType};

pub fn create_gamepad() {
    let mut gilrs = Gilrs::new().unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;

    loop {
        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(Button::DPadDown, _) => {
                    println!("{:?} New event from {}: Button South is pressed", time, id);
                }
                _ => (),
            }
            //println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);
        }

        // You can also use cached gamepad state
        // if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
        //     if gamepad.is_pressed(Button::South) {
        //         println!("Button South is pressed (XBox - A, PS - X)");
        //     }
        // }
    }
}
