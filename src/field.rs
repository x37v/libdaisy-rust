use hal::prelude::*;
use stm32h7xx_hal as hal;

type I2CWrite = dyn hal::hal::blocking::i2c::Write<Error = hal::i2c::Error>;

const BASE_ADDR: u8 = 0b01000000;
const LED_ADDR0: u8 = BASE_ADDR | 0x00;
const LED_ADDR1: u8 = BASE_ADDR | 0x02;

const PCA9685_MODE1: u8 = 0x00; // location for Mode1 register address
const PCA9685_MODE2: u8 = 0x01; // location for Mode2 reigster address
const PCA9685_LED0: u8 = 0x06; // location for start of LED0 registers
const PRE_SCALE_MODE: u8 = 0xFE; //location for setting prescale (clock speed)

const PCA9685_AUTO_INC: u8 = 0b0010_0000;

const PCA9685_INV: u8 = 0b0001_0000;

const GAMMA: [u16; 256] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 8, 9, 10,
    11, 12, 13, 15, 16, 17, 18, 20, 21, 23, 25, 26, 28, 30, 32, 34, 36, 38, 40, 43, 45, 48, 50, 53,
    56, 59, 62, 65, 68, 71, 75, 78, 82, 85, 89, 93, 97, 101, 105, 110, 114, 119, 123, 128, 133,
    138, 143, 149, 154, 159, 165, 171, 177, 183, 189, 195, 202, 208, 215, 222, 229, 236, 243, 250,
    258, 266, 273, 281, 290, 298, 306, 315, 324, 332, 341, 351, 360, 369, 379, 389, 399, 409, 419,
    430, 440, 451, 462, 473, 485, 496, 508, 520, 532, 544, 556, 569, 582, 594, 608, 621, 634, 648,
    662, 676, 690, 704, 719, 734, 749, 764, 779, 795, 811, 827, 843, 859, 876, 893, 910, 927, 944,
    962, 980, 998, 1016, 1034, 1053, 1072, 1091, 1110, 1130, 1150, 1170, 1190, 1210, 1231, 1252,
    1273, 1294, 1316, 1338, 1360, 1382, 1404, 1427, 1450, 1473, 1497, 1520, 1544, 1568, 1593, 1617,
    1642, 1667, 1693, 1718, 1744, 1770, 1797, 1823, 1850, 1877, 1905, 1932, 1960, 1988, 2017, 2045,
    2074, 2103, 2133, 2162, 2192, 2223, 2253, 2284, 2315, 2346, 2378, 2410, 2442, 2474, 2507, 2540,
    2573, 2606, 2640, 2674, 2708, 2743, 2778, 2813, 2849, 2884, 2920, 2957, 2993, 3030, 3067, 3105,
    3143, 3181, 3219, 3258, 3297, 3336, 3376, 3416, 3456, 3496, 3537, 3578, 3619, 3661, 3703, 3745,
    3788, 3831, 3874, 3918, 3962, 4006, 4050, 4095,
];

pub struct Field {
    //led_i2c: Pca9685<I2c<I2C1>>,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Led {
    on: u16,
    off: u16,
}

impl Default for Led {
    fn default() -> Self {
        //full off
        Self { on: 0, off: 0x1001 }
    }
}

#[repr(C, packed)]
struct LedTxBuffer {
    reg: u8,
    leds: [Led; 16],
}

impl Default for LedTxBuffer {
    fn default() -> Self {
        let leds: [Led; 16] = Default::default();
        Self {
            reg: PCA9685_LED0,
            leds,
        }
    }
}

pub struct LedDriver {
    addr: u8,
    buffer: LedTxBuffer,
}

//TODO DMA
impl LedDriver {
    pub fn new(i2c: &mut I2CWrite, addr: u8) -> Self {
        //configure, copied from libDaisy
        //mode 1:
        //  auto increment
        //mode 2:
        //  OE-high = high Impedance
        //  Push-Pull outputs
        //  outputs change on STOP
        //  outputs inverted
        i2c.write(addr, &[PCA9685_MODE1, PCA9685_AUTO_INC, 0b0011_0110])
            .unwrap();
        //turn all, full off
        i2c.write(addr, &[0xFA, 0, 0, 0, 0x10]).unwrap();

        Self {
            addr,
            buffer: Default::default(),
        }
    }

    pub fn set_all(&mut self, brightness: u8) {
        let cycles = GAMMA[brightness as usize];

        //full off
        if cycles == 0 {
            for (index, mut led) in self.buffer.leds.iter_mut().enumerate() {
                let on = (index << 2) as u16; //offset on times
                led.on = on;
                led.off = 0x1001 + on;
            }
        } else {
            for (index, mut led) in self.buffer.leds.iter_mut().enumerate() {
                let on = (index << 2) as u16; //offset on times
                led.on = if cycles >= 0x0FFF { 0x1000 | on } else { on };
                led.off = on.saturating_add(cycles) & 0x0FFF;
            }
        }
    }

    pub fn set(&mut self, index: usize, brightness: u8) {
        assert!(index < 16);
        let cycles = GAMMA[brightness as usize];

        let led: &mut Led = &mut self.buffer.leds[index];

        let on = (index << 2) as u16; //offset on times
                                      //full off
        if cycles == 0 {
            led.on = on;
            led.off = 0x1001 + on;
        } else {
            led.on = if cycles >= 0x0FFF { 0x1000 | on } else { on };
            led.off = on.saturating_add(cycles) & 0x0FFF;
        }
    }

    pub fn draw(&self, i2c: &mut I2CWrite) {
        i2c.write(self.addr, unsafe {
            core::slice::from_raw_parts(
                core::mem::transmute::<_, *const u8>(&self.buffer),
                core::mem::size_of::<LedTxBuffer>(),
            )
        })
        .unwrap();
    }
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

        let mut drivers = [
            LedDriver::new(&mut led_i2c, LED_ADDR0),
            LedDriver::new(&mut led_i2c, LED_ADDR1),
        ];

        drivers[0].set(2, 0xFF);
        drivers[0].set(14, 0x7F);
        drivers[1].set_all(0xAF);

        drivers[0].draw(&mut led_i2c);
        drivers[1].draw(&mut led_i2c);

        /*
        //configure, copied from libDaisy
        //mode 1:
        //  auto increment
        //mode 2:
        //  OE-high = high Impedance
        //  Push-Pull outputs
        //  outputs change on STOP
        //  outputs inverted
        for a in &[LED_ADDR0, LED_ADDR1] {
            led_i2c
                .write(*a, &[PCA9685_MODE1, PCA9685_AUTO_INC, 0b0011_0110])
                .unwrap();
            //turn all, full off
            led_i2c.write(*a, &[0xFA, 0, 0, 0, 0x10]).unwrap();
        }

        //LSB, MSB
        led_i2c
            .write(LED_ADDR0, &[0x06, 0xFF, 0x1F, 0x0, 0x0])
            .unwrap();

        led_i2c
            .write(LED_ADDR1, &[0x1E, 0x00, 0x10, 0x0, 0x0])
            .unwrap();
        */

        Self {}
    }
}
