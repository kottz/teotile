#![no_std]
#![no_main]

extern crate alloc;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::dma::{AnyChannel, Channel};
use embassy_rp::gpio::Pin;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{
    Common, Config, FifoJoin, Instance, InterruptHandler, Pio, PioPin, ShiftConfig, ShiftDirection,
    StateMachine,
};
use embassy_rp::{bind_interrupts, clocks, into_ref, Peripheral, PeripheralRef};
use embassy_time::{Duration, Ticker, Timer};
use fixed::types::U24F8;
use fixed_macro::fixed;
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

use core::mem::MaybeUninit;
use core::time::Duration as StdDuration;
use teotile::{ButtonState, CommandType, GameCommand, GameEngine, Player};

mod gamepad;
use gamepad::{GamepadEvent, GamepadHandler};

use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 32768;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

pub struct Ws2812<'d, P: Instance, const S: usize, const N: usize> {
    dma: PeripheralRef<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize, const N: usize> Ws2812<'d, P, S, N> {
    pub fn new(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'd,
        pin: impl PioPin,
    ) -> Self {
        into_ref!(dma);

        // Setup sm0
        let side_set = pio::SideSet::new(false, 1, false);
        let mut a: pio::Assembler<32> = pio::Assembler::new_with_side_set(side_set);

        const T1: u8 = 2; // start bit
        const T2: u8 = 5; // data bit
        const T3: u8 = 3; // stop bit
        const CYCLES_PER_BIT: u32 = (T1 + T2 + T3) as u32;

        let mut wrap_target = a.label();
        let mut wrap_source = a.label();
        let mut do_zero = a.label();
        a.set_with_side_set(pio::SetDestination::PINDIRS, 1, 0);
        a.bind(&mut wrap_target);
        // Do stop bit
        a.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
        // Do start bit
        a.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
        // Do data bit = 1
        a.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut wrap_target, T2 - 1, 1);
        a.bind(&mut do_zero);
        // Do data bit = 0
        a.nop_with_delay_and_side_set(T2 - 1, 0);
        a.bind(&mut wrap_source);

        let prg = a.assemble_with_wrap(wrap_source, wrap_target);
        let mut cfg = Config::default();

        // Pin config
        let out_pin = pio.make_pio_pin(pin);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);

        cfg.use_program(&pio.load_program(&prg), &[&out_pin]);

        // Clock config, measured in kHz to avoid overflows
        let clock_freq = U24F8::from_num(clocks::clk_sys_freq() / 1000);
        let ws2812_freq = fixed!(800: U24F8);
        let bit_freq = ws2812_freq * CYCLES_PER_BIT;
        cfg.clock_divider = clock_freq / bit_freq;

        // FIFO config
        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 24,
            direction: ShiftDirection::Left,
        };

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            dma: dma.map_into(),
            sm,
        }
    }

    pub async fn write(&mut self, colors: &[RGB8; N]) {
        // Precompute the word bytes from the colors
        let mut words = [0u32; N];
        for i in 0..N {
            let word = (u32::from(colors[i].g) << 24)
                | (u32::from(colors[i].r) << 16)
                | (u32::from(colors[i].b) << 8);
            words[i] = word;
        }

        // DMA transfer
        self.sm.tx().dma_push(self.dma.reborrow(), &words).await;

        Timer::after_micros(55).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the allocator
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }

    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    // Initialize GamepadHandler with GPIO pins
    let mut gamepad = GamepadHandler::new(
        p.PIN_2.degrade(),
        p.PIN_3.degrade(),
        p.PIN_4.degrade(),
        p.PIN_5.degrade(),
        p.PIN_6.degrade(),
        p.PIN_7.degrade(),
        p.PIN_8.degrade(),
        p.PIN_9.degrade(),
        p.PIN_10.degrade(),
        p.PIN_11.degrade(),
        p.PIN_12.degrade(),
        p.PIN_13.degrade(),
    );

    // This is the number of leds in the string.
    const NUM_LEDS: usize = 144; // Adjusted to 12x12 grid
    let mut data = [RGB8::default(); NUM_LEDS];

    let mut ws2812 = Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_16);

    let mut game_engine = GameEngine::default();

    // Main game loop
    let mut ticker = Ticker::every(Duration::from_millis(16)); // ~60 FPS
    loop {
        // Poll for gamepad events
        gamepad.poll_events().await;

        while let Some(event) = gamepad.get_event() {
            if let Some(command) = gamepad_event_to_command(event) {
                let _ = game_engine.process_input(command);
            }
        }

        // Update game state
        game_engine.update(StdDuration::from_millis(16)).unwrap();

        // Render game state
        if let Ok(render_board) = game_engine.render() {
            // Convert render_board to LED data
            for col in 0..12 {
                for row in 0..12 {
                    let i = col + row * 12;
                    let pixel = render_board.get(col, row);
                    data[i] = RGB8::new(pixel.r, pixel.g, pixel.b);
                }
            }
            ws2812.write(&data).await;
        }

        ticker.next().await;
    }
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
    }
}

fn player_from_id(id: usize) -> Player {
    match id {
        0 => Player::Player1,
        1 => Player::Player2,
        _ => Player::Player1, // Default to Player1 for any additional controllers
    }
}
