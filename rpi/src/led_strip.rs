use crate::output::Output;
use anyhow::{Context, Result};
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};
use teotile::RGB;

pub struct LedStrip {
    controller: Controller,
    led_count: usize,
}

impl LedStrip {
    pub fn new(pin: i32, led_count: i32) -> Result<Self> {
        let controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(
                0,
                ChannelBuilder::new()
                    .pin(pin)
                    .count(led_count)
                    .strip_type(StripType::Ws2812)
                    .brightness(20)
                    .build(),
            )
            .build()
            .context("Failed to initialize LED strip controller")?;

        Ok(Self {
            controller,
            led_count: led_count as usize,
        })
    }
}

impl Output for LedStrip {
    fn render(&mut self, render_board: &teotile::RenderBoard) -> Result<()> {
        let leds = self.controller.leds_mut(0);

        for (i, led) in leds.iter_mut().enumerate() {
            if i >= self.led_count {
                break;
            }

            let row = i / 12;
            let col = if row % 2 == 0 { i % 12 } else { 11 - (i % 12) };

            let color: RGB = render_board.get(col, row);
            *led = [color.b, color.g, color.r, 0];
        }

        self.controller
            .render()
            .context("Failed to render LED strip")?;
        Ok(())
    }
}
