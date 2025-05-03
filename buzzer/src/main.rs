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
    
    // Define maximum duty cycle
    let max_duty = u16::MAX;
    
    // Define the PWM for the buzzer using the PWM_SLICE4 and PIN_8
    let mut pwm_buzzer = Pwm::new_output_a(p.PWM_SLICE4, p.PIN_8, pwm::Config::default());
    
    // frequency for the buzzer (e.g., 1kHz)
    let frequency = 1000; // 1000 Hz (1 kHz)
    pwm_buzzer.set_duty_cycle(0).expect("Failed to set duty cycle");
    
    loop {
        // Sound at a low volume (duty cycle 50%)
        pwm_buzzer.set_duty_cycle(max_duty / 2).expect("Failed to set duty cycle");
        Timer::after(Duration::from_secs(1)).await;
        
        // Silence
        pwm_buzzer.set_duty_cycle(0).expect("Failed to set duty cycle");
        Timer::after(Duration::from_secs(1)).await;
        
        // Sound at full volume (duty cycle 100%)
        pwm_buzzer.set_duty_cycle(max_duty).expect("Failed to set duty cycle");
        Timer::after(Duration::from_secs(1)).await;
        
        // Silence again
        pwm_buzzer.set_duty_cycle(0).expect("Failed to set duty cycle");
        Timer::after(Duration::from_secs(1)).await;
    }
}

