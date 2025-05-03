#![no_std]
#![no_main]
use embassy_executor::Spawner;
use embassy_rp::pwm::{self, Pwm};
use embassy_time::{Duration, Timer};
use embedded_hal_1::pwm::SetDutyCycle;
use {defmt_rtt as _, panic_probe as _};
use defmt::info;
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let max_duty = u16::MAX;
    info!("Max duty cycle: {}", max_duty);
    
    let mut pwm_red = Pwm::new_output_b(p.PWM_SLICE4, p.PIN_9, pwm::Config::default());
    let mut pwm_green = Pwm::new_output_a(p.PWM_SLICE5, p.PIN_10, pwm::Config::default());
    let mut pwm_blue = Pwm::new_output_a(p.PWM_SLICE6, p.PIN_12, pwm::Config::default());
    
    loop {
        // Red
        pwm_red.set_duty_cycle(max_duty / 30).expect("TODO: panic message"); // 50% duty cycle
        pwm_green.set_duty_cycle(0).expect("TODO: panic message");
        pwm_blue.set_duty_cycle(0).expect("TODO: panic message");
        Timer::after(Duration::from_secs(2)).await;
        
        // Green
        pwm_red.set_duty_cycle(0).expect("TODO: panic message");
        pwm_green.set_duty_cycle(max_duty / 2).expect("TODO: panic message"); // 50% duty cycle
        pwm_blue.set_duty_cycle(0).expect("TODO: panic message");
        Timer::after(Duration::from_secs(2)).await;
        
        // Blue
        pwm_red.set_duty_cycle(0).expect("TODO: panic message");
        pwm_green.set_duty_cycle(0).expect("TODO: panic message");
        pwm_blue.set_duty_cycle(max_duty / 2).expect("TODO: panic message"); // 50% duty cycle
        Timer::after(Duration::from_secs(2)).await;
    }
}