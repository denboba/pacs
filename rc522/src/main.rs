#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Output, Level};
use embassy_rp::peripherals::{SPI0, DMA_CH0, DMA_CH1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6};
use embassy_rp::spi::{self, Spi};
use embassy_time::{Duration, Timer};
use embedded_hal_async::spi::SpiBus;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize RP2350 peripherals
    let p = embassy_rp::init(Default::default());
    
    // Configure the SPI bus
    let mut config = spi::Config::default();
    config.frequency = 1_000_000;
    config.phase = spi::Phase::CaptureOnFirstTransition;
    config.polarity = spi::Polarity::IdleLow;
    
    // Initialize the SPI pins
    let clk = p.PIN_2;   // SCK (GP2)
    let mosi = p.PIN_3;  // MOSI (GP3)
    let miso = p.PIN_4;  // MISO (GP4)
    
    // Create SPI instance with proper type parameters
    let mut spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, config);
    
    // RC522 control pins
    let mut cs = Output::new(p.PIN_5, Level::High);  // SDA/SS (GP5)
    let mut rst = Output::new(p.PIN_6, Level::High); // RST (GP6)
    
    // Reset the RC522
    rst.set_low();
    Timer::after(Duration::from_millis(10)).await;
    rst.set_high();
    Timer::after(Duration::from_millis(10)).await;
    
    // RC522 register addresses
    const COMMAND_REG: u8 = 0x01;
    const FIFO_DATA_REG: u8 = 0x09;
    const FIFO_LEVEL_REG: u8 = 0x0A;
    const IRQ_REG: u8 = 0x04;
    const ERROR_REG: u8 = 0x06;
    const VERSION_REG: u8 = 0x37;
    
    // RC522 commands
    const IDLE_CMD: u8 = 0x00;
    const TRANSCEIVE_CMD: u8 = 0x0C;
    
    // Helper function to write to RC522 register
    async fn write_register(
        spi: &mut Spi<'_, SPI0,  spi::Async>,
        cs: &mut Output<'_>,
        reg: u8,
        value: u8,
    ) {
        cs.set_low();
        let tx_buf = [(reg << 1) & 0x7E, value];
        let mut rx_buf = [0u8; 2];
        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        cs.set_high();
    }
    
    // Helper function to read from RC522 register
    async fn read_register(
        spi: &mut Spi<'_, SPI0,  spi::Async>,
        cs: &mut Output<'_>,
        reg: u8,
    ) -> u8 {
        cs.set_low();
        let tx_buf = [((reg << 1) & 0x7E) | 0x80, 0x00];
        let mut rx_buf = [0u8; 2];
        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        cs.set_high();
        rx_buf[1]
    }
    
    // Initialize RC522
    write_register(&mut spi, &mut cs, COMMAND_REG, IDLE_CMD).await;
    write_register(&mut spi, &mut cs, 0x2A, 0x8D).await;
    write_register(&mut spi, &mut cs, 0x2B, 0x3E).await;
    write_register(&mut spi, &mut cs, 0x2D, 30).await;
    write_register(&mut spi, &mut cs, 0x2C, 0).await;
    write_register(&mut spi, &mut cs, 0x15, 0x40).await;
    write_register(&mut spi, &mut cs, 0x11, 0x3D).await;
    
    // Read version to verify communication
    let version = read_register(&mut spi, &mut cs, VERSION_REG).await;
    info!("RC522 version: 0x{:x}", version);
    
    loop {
        // Check for cards
        write_register(&mut spi, &mut cs, COMMAND_REG, IDLE_CMD).await;
        write_register(&mut spi, &mut cs, 0x0F, 0x10).await;
        write_register(&mut spi, &mut cs, 0x0D, 0x00).await;
        write_register(&mut spi, &mut cs, 0x01, 0x0C).await;
        write_register(&mut spi, &mut cs, COMMAND_REG, TRANSCEIVE_CMD).await;
        write_register(&mut spi, &mut cs, 0x0D, 0x00).await;
        
        // Send REQA command
        cs.set_low();
        let tx_buf = [FIFO_DATA_REG, 0x26];
        let mut rx_buf = [0u8; 2];
        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        cs.set_high();
        
        Timer::after(Duration::from_millis(10)).await;
        
        // Check if card detected
        let irq = read_register(&mut spi, &mut cs, IRQ_REG).await;
        let error = read_register(&mut spi, &mut cs, ERROR_REG).await;
        
        if irq & 0x01 == 0x01 && error == 0x00 {
            // Card detected, read UID
            let mut uid = [0u8; 10];
            cs.set_low();
            let tx_buf = [FIFO_DATA_REG, 0x93, 0x20];
            let mut rx_buf = [0u8; 3];
            spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
            cs.set_high();
            
            // Read UID bytes
            let uid_len = read_register(&mut spi, &mut cs, FIFO_LEVEL_REG).await;
            if uid_len > 0 && uid_len <= 10 {
                cs.set_low();
                let tx_buf = [FIFO_DATA_REG | 0x80];
                let mut rx_buf = [0u8; 11];
                let mut full_tx = [0u8; 11];
                full_tx[0] = tx_buf[0];
                spi.transfer(&mut rx_buf[..(1 + uid_len as usize)], &full_tx[..(1 + uid_len as usize)]).await.unwrap();
                cs.set_high();
                // Extract UID
                for i in 0..uid_len as usize {
                    uid[i] = rx_buf[i + 1];
                }
                info!("Card detected! UID: {:x}", &uid[..uid_len as usize]);
            } else if uid_len == 0 {
                info!("No card detected");
            } else {
                info!("Invalid UID length: {}", uid_len);
            }
            if uid_len <= 10 {
                cs.set_low();
                let tx_buf = [FIFO_DATA_REG | 0x80];
                let mut rx_buf = [0u8; 11];
                let mut full_tx = [0u8; 11];
                full_tx[0] = tx_buf[0];
                spi.transfer(&mut rx_buf[..(1 + uid_len as usize)], &full_tx[..(1 + uid_len as usize)]).await.unwrap();
                cs.set_high();
                // Extract UID
                for i in 0..uid_len as usize {
                    uid[i] = rx_buf[i + 1];
                }
                info!("Card detected! UID: {:x}", &uid[..uid_len as usize]);
            }
        } else {
            info!("No card detected");
        }
        
        Timer::after(Duration::from_millis(500)).await;
    }
}