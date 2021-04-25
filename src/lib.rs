#![no_std]
#![allow(dead_code)]

// #[macro_use(singleton)]
// extern crate cortex_m;

use cortex_m::asm::delay as delay_cycles;

use stm32h7xx_hal::time::{Hertz, MegaHertz};

pub const MILLI: u32 = 1_000;
pub const MICRO: u32 = 1_000_000;
pub const NANO: u32 = 1_000_000_000;

pub const AUDIO_FRAME_RATE_HZ: u32 = 1_000;
pub const AUDIO_BLOCK_SIZE: u16 = 48;
pub const AUDIO_SAMPLE_RATE: usize = 48_000;
pub const AUDIO_SAMPLE_HZ: Hertz = Hertz(48_000);
pub const CLOCK_RATE_HZ: Hertz = Hertz(480_000_000_u32);

pub const MILICYCLES: u32 = CLOCK_RATE_HZ.0 / MILLI;
pub const MICROCYCLES: u32 = CLOCK_RATE_HZ.0 / MICRO;

pub type FrameTimer = stm32h7xx_hal::timer::Timer<stm32h7xx_hal::stm32::TIM2>;

pub mod audio;
pub mod gpio;
pub mod hid;
pub mod logger;
pub mod mpu;
pub mod prelude;
pub mod sdram;
pub mod system;

pub mod field;

/// Delay for ms, note if interrupts are active delay time will extend
pub fn delay_ms(ms: u32) {
    delay_cycles(ms * MILICYCLES);
}

/// Delay for micro sec, note if interrupts are active delay time will extend
pub fn delay_us(us: u32) {
    delay_cycles(us * MICROCYCLES);
}

/// Delay for nano sec, note if interrupts are active delay time will extend
pub fn delay_ns(ns: u32) {
    let cycles = CLOCK_RATE_HZ.0.saturating_mul(ns) / MICRO;
    delay_cycles(if cycles == 0 { 1 } else { cycles });
}

// pub fn ms_to_cycles(ms: u32) {

// }
