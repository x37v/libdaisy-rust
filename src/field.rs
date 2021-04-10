use hal::{
    device::I2C1,
    i2c::{I2c, Stop},
    prelude::*,
};
use stm32h7xx_hal as hal;

const LED_ADDR0: u8 = 0x00;
const LED_ADDR1: u8 = 0x02;

pub struct Field {
    led_i2c: I2c<I2C1>,
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
        //led_i2c.tx_dma(true);
        led_i2c.master_write(LED_ADDR0, 1, Stop::Automatic);
        let _ = led_i2c.write(LED_ADDR0, &[0xF0]);
        //let _ = led_i2c.write(LED_ADDR1, &[0xFF]);
        Self { led_i2c }
    }
}
