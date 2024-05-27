//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::{entry, hal::{Timer, prelude::_rphal_pio_PIOExt}};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;

use ws2812_pio::Ws2812;
use embedded_hal::digital::v2::InputPin;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

include!("../yinyang_pixels.rs");
const STRIP_LEN: usize = 144;
#[derive(defmt::Format, Debug)]
struct Test {
    a: u32,
    b: u32,
}

fn _coordinate_to_index(x: usize, y: usize, width: usize, _height: usize) -> usize {
    let index = if y % 2 == 0 {
        // For even rows, counting from left to right
        y * width + x
    } else {
        // For odd rows, counting from right to left (because of the zig-zag pattern, every other
        // row of leds is flipped)
        y * width + (width - 1 - x)
    };
    index
}

pub fn idx(x: usize, y: usize) -> usize {
    _coordinate_to_index(x, y, 12, 12)
}

pub fn coordinate_to_index(x: usize, y: usize) -> usize {
    let index = if y % 2 == 0 {
        // For even rows, counting from left to right
        y * 12 + x
    } else {
        // For odd rows, counting from right to left
        y * 12 + (11 - x)
    };

    index
}
use smart_leds::{brightness, SmartLedsWrite, RGB8};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut ws = Ws2812::new(pins.gpio14.into_mode(), &mut pio, sm0, clocks.peripheral_clock.freq(), timer.count_down());
    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead. If you have
    // a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here.
    let mut led_pin = pins.led.into_push_pull_output();
    let button_pin = pins.gpio16.into_pull_up_input();

    use smart_leds::{SmartLedsWrite, RGB8, brightness};

    let mut leds: [RGB8; STRIP_LEN] = [(0, 0, 0).into(); STRIP_LEN];

    // Bring down the overall brightness of the strip to not blow
    // the USB power supply: every LED draws ~60mA, RGB means 3 LEDs per
    // ws2812 LED, for 3 LEDs that would be: 3 * 3 * 60mA, which is
    // already 540mA for just 3 white LEDs!
    let strip_brightness = 8u8; // Limit brightness x/256

    loop {
        //leds.iter_mut().for_each(|x| *x = (255, 120, 0).into());
        //ws.write(brightness(leds.iter().copied(), strip_brightness)).unwrap();
        //let color: RGB8 = (255, 0, 0).into();
        //ws.write([color].iter().copied()).unwrap();
        info!("on!");
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        //let color: RGB8 = (0, 0, 0).into();
        // update all values of the strip
        leds.iter_mut().for_each(|x| *x = (255, 0, 0).into());

        
        //ws.write(brightness(leds.iter().copied(), strip_brightness)).unwrap();
        for i in 0..STRIP_LEN {
            let index = idx(i % 12, i % 12);
            leds[index] = (3, 178, 58).into();
            let index = idx(i % 12, 11 - i % 12);
            leds[index] = (3, 178, 58).into();
            //if i % 24 == 0 {
            //    leds[i] = (255, 122, 0).into();
            //} else {
                //leds[i] = (0, 0, 255).into();
            //}
            //leds[i] = (0, 0, 255).into();
            ws.write(brightness(leds.iter().copied(), strip_brightness)).unwrap();
            delay.delay_ms(10);
        }
        for i in (0..STRIP_LEN).rev() {
            leds[i] = (0, 255, 0).into();
            ws.write(brightness(leds.iter().copied(), strip_brightness)).unwrap();
            delay.delay_ms(1);
        }
        let test = Test { a: 23, b: 344 };
        let a = 23;
        info!("off! {} {:?} {:?}",a, 344, test);
        if button_pin.is_low().unwrap() {
            info!("button pressed");
            for (i, (r,g,b)) in PIXEL_VALUES.iter().enumerate() {
                leds[i] = (*r, *g, *b).into();
            }
            ws.write(brightness(leds.iter().copied(), strip_brightness)).unwrap();
            delay.delay_ms(20);
            while button_pin.is_low().unwrap() {
                delay.delay_ms(20);
            }
        } else {
            info!("button not pressed");
            delay.delay_ms(500);
            led_pin.set_low().unwrap();
        }

    }
}

// End of file
