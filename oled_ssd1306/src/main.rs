#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{Config as I2cConfig, I2c, InterruptHandler as I2CInterruptHandler};
use embassy_rp::peripherals::I2C0;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use {defmt_rtt as _, panic_probe as _};
bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
});
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    
    // Pins for Pico 2W
    let sda = p.PIN_16;
    let scl = p.PIN_17;
    
    // Async I2C with DMA
    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, I2cConfig::default());
    
    
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
        .into_buffered_graphics_mode();
    
    if let Err(e) = display.init(){
        // defmt::error!("Display init error: {:?}", e);
        return;
    }
    
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    
    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Middle)
        .draw(&mut display)
        .unwrap();
    
    Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    
    if let Err(e) = display.flush() {
    
    }
    
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
