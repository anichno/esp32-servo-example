use anyhow::Result;
use embedded_hal::{digital::v2::ToggleableOutputPin, prelude::*};
use esp_idf_hal::ledc::{
    config::{Resolution, TimerConfig},
    Channel, Timer,
};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use num::clamp;

const SERVO_MIN_PULSEWIDTH_US: u32 = 300;
const SERVO_MAX_PULSEWIDTH_US: u32 = 2200;
const SERVO_MAX_ANGLE: f32 = 180.0;
const PWM_PERIOD: u32 = 50;
const TIMER_RESOLUTION: u32 = 65535;

const US_TO_DUTY: f32 = (TIMER_RESOLUTION as f32) / (1000.0 / (PWM_PERIOD as f32) * 1000.0);

fn angle_to_duty(angle: f32) -> u32 {
    assert!((0.0..=SERVO_MAX_ANGLE).contains(&angle));

    (clamp(
        angle / SERVO_MAX_ANGLE * ((SERVO_MAX_PULSEWIDTH_US - SERVO_MIN_PULSEWIDTH_US) as f32)
            + SERVO_MIN_PULSEWIDTH_US as f32,
        SERVO_MIN_PULSEWIDTH_US as f32,
        SERVO_MAX_PULSEWIDTH_US as f32,
    ) * US_TO_DUTY) as u32
}

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut led = peripherals.pins.gpio2.into_output()?;

    let config = TimerConfig::default()
        .frequency(PWM_PERIOD.Hz())
        .resolution(Resolution::Bits16);
    let timer = Timer::new(peripherals.ledc.timer0, &config)?;
    let mut channel = Channel::new(peripherals.ledc.channel0, &timer, peripherals.pins.gpio32)?;

    channel.set_duty(angle_to_duty(0.0))?;

    let mut delay = esp_idf_hal::delay::FreeRtos;
    let mut zero = true;
    loop {
        led.toggle()?;
        delay.delay_ms(2000u32);

        if zero {
            channel.set_duty(angle_to_duty(180.0))?;
        } else {
            channel.set_duty(angle_to_duty(0.0))?;
        }
        zero = !zero;
    }
}
