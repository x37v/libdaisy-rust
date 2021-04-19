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

const PCA9685_AUTO_INC: u8 = 0b0010_0000;

const PCA9685_INV: u8 = 0b0001_0000;

pub struct Field {
    //led_i2c: Pca9685<I2c<I2C1>>,
}

impl Field {
    pub fn new(
        i2cd: hal::stm32::I2C1,
        i2crec: hal::rcc::rec::I2c1,
        scl: hal::gpio::gpiob::PB8<hal::gpio::Analog>,
        sda: hal::gpio::gpiob::PB9<hal::gpio::Analog>,
        clocks: &hal::rcc::CoreClocks,
    ) -> Self {
        let mut led_i2c = i2cd.i2c(
            (
                scl.into_alternate_af4().set_open_drain(),
                sda.into_alternate_af4().set_open_drain(),
            ),
            1.mhz(),
            i2crec,
            clocks,
        );
        crate::delay_ms(20);
        led_i2c.write(LED_ADDR0, &[PCA9685_MODE1, 0]).unwrap();
        crate::delay_ms(20);
        led_i2c.write(LED_ADDR0, &[PCA9685_MODE1, 0]).unwrap();
        crate::delay_ms(20);
        led_i2c
            .write(LED_ADDR0, &[PCA9685_MODE1, PCA9685_AUTO_INC])
            .unwrap();
        crate::delay_ms(20);
        led_i2c
            .write(LED_ADDR0, &[PCA9685_MODE2, 0b0011_0110])
            .unwrap();
        crate::delay_ms(20);

        //LSB, MSB
        led_i2c.write(LED_ADDR0, &[0x06, 0xFF, 0x1F, 0, 0]).unwrap();
        crate::delay_ms(20);
        led_i2c.write(LED_ADDR0, &[0x0A, 0xFF, 0x1F, 0, 0]).unwrap();

        /*
        led_i2c
            .set_output_change_behavior(pwm_pca9685::OutputStateChange::OnStop)
            .unwrap();
        led_i2c
            .set_output_logic_state(pwm_pca9685::OutputLogicState::Inverted)
            .unwrap();
        led_i2c
            .set_output_driver(pwm_pca9685::OutputDriver::TotemPole)
            .unwrap();
        led_i2c
            .set_disabled_output_value(pwm_pca9685::DisabledOutputValue::HighImpedance)
            .unwrap();
        led_i2c.enable().unwrap();

        led_i2c.set_channel_full_on(Channel::All, 4095).unwrap();

        led_i2c.set_channel_full_on(Channel::C0, 4095).unwrap();
        led_i2c.set_channel_full_on(Channel::C2, 4095).unwrap();
        led_i2c.set_channel_full_on(Channel::C14, 128).unwrap();

        led_i2c.set_address(LED_ADDR1).unwrap();
        led_i2c.set_channel_full_off(Channel::All).unwrap();
        led_i2c.set_channel_full_on(Channel::C1, 4095).unwrap();
        led_i2c.set_channel_full_on(Channel::C3, 4095).unwrap();

        led_i2c.set_address(LED_ADDR0).unwrap();
        led_i2c.set_channel_full_on(Channel::C9, 4095).unwrap();
        led_i2c.set_channel_full_on(Channel::C15, 4095).unwrap();
        */

        Self {}
    }
}
