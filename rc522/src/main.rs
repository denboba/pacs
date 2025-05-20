#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Output, Level};
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::{self, Spi};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use embedded_hal_bus::spi::ExclusiveDevice;
use mfrc522::{comm::blocking::spi::SpiInterface, Mfrc522};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize RP2350 peripherals
    let p = embassy_rp::init(Default::default());
    
    // Configure the SPI bus
    let mut config = spi::Config::default();
    config.frequency = 1_000_000;
    // Initialize the SPI pins
    let clk = p.PIN_2;   // SCK (GP2)
    let mosi = p.PIN_3;  // MOSI (GP3)
    let miso = p.PIN_4;  // MISO (GP4)
    
    // Create SPI instance with proper type parameters
    let mut spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, config);
    
    // RC522 control pins
    let mut cs = Output::new(p.PIN_5, Level::High);  // SDA/SS (GP5)
    let delay = embassy_time::Delay;
    let spi = ExclusiveDevice::new(spi, cs, delay).unwrap();
    
    let itf = SpiInterface::new(spi);
    let mut rfid = Mfrc522::new(itf).init().unwrap();
    
    loop {
        if let Ok(atqa) = rfid.reqa() {
            if let Ok(uid) = rfid.select(&atqa) {
                info!("Card detected! uid: {:?}", uid.as_bytes());
                embassy_time::Timer::after_millis(500).await;
            }
        }
    }
    
   
}