use core::convert::Into;
use hal::{
    device::I2C1,
    i2c::{I2c, Stop},
    prelude::*,
};
use stm32h7xx_hal as hal;

use pwm_pca9685::{Address, Channel, Pca9685};

const BASE_ADDR: u8 = 0b01000000;
const LED_ADDR0: u8 = BASE_ADDR | 0x00;
const LED_ADDR1: u8 = BASE_ADDR | 0x02;

const PCA9685_MODE1: u8 = 0x00; // location for Mode1 register address
const PCA9685_MODE2: u8 = 0x01; // location for Mode2 reigster address
const PCA9685_LED0: u8 = 0x06; // location for start of LED0 registers
const PRE_SCALE_MODE: u8 = 0xFE; //location for setting prescale (clock speed)

//XXX libDaisy shifts left by 1
const PCA9685_AUTO_INC: u8 = 0x10;

pub struct Field {
    led_i2c: Pca9685<I2c<I2C1>>,
}

impl Field {
    pub fn new(
        i2cd: hal::stm32::I2C1,
        i2crec: hal::rcc::rec::I2c1,
        scl: hal::gpio::gpiob::PB8<hal::gpio::Analog>,
        sda: hal::gpio::gpiob::PB9<hal::gpio::Analog>,
        clocks: &hal::rcc::CoreClocks,
    ) -> Self {
        //pwm_pca9685::Address::default(),
        let addr: Address = Address::default();
        let mut led_i2c = Pca9685::new(
            i2cd.i2c(
                (
                    scl.into_alternate_af4().set_open_drain(),
                    sda.into_alternate_af4().set_open_drain(),
                ),
                1.mhz(),
                i2crec,
                clocks,
            ),
            addr,
        )
        .unwrap();
        led_i2c.reset_internal_driver_state();
        led_i2c.enable().unwrap();
        led_i2c
            .set_output_logic_state(pwm_pca9685::OutputLogicState::Inverted)
            .unwrap();
        led_i2c
            .set_output_driver(pwm_pca9685::OutputDriver::TotemPole)
            .unwrap();

        let on = [0; 16];
        let off = [0; 16];
        led_i2c.set_all_on_off(&on, &off).unwrap();

        led_i2c.set_channel_full_on(Channel::C0, 4095).unwrap();
        led_i2c.set_channel_full_on(Channel::C4, 4095).unwrap();
        led_i2c.set_channel_full_on(Channel::C14, 128).unwrap();

        Self { led_i2c }
    }
}
