use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::{TextStyle, TextStyleBuilder},
};

use hal::prelude::*;
use shift::{Delay as ShiftDelay, ShiftClockDelay, ShiftIn};
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

pub type FieldSwitches = (
    hal::gpio::gpiob::PB15<hal::gpio::Input<hal::gpio::PullUp>>,
    hal::gpio::gpiob::PB14<hal::gpio::Input<hal::gpio::PullUp>>,
);

pub type FieldGates = (
    hal::gpio::gpiob::PB12<hal::gpio::Input<hal::gpio::Floating>>,
    hal::gpio::gpioc::PC0<hal::gpio::Output<hal::gpio::PushPull>>,
);

pub struct FieldLeds {
    i2c: hal::i2c::I2c<hal::stm32::I2C1>,
    drivers: [LedDriver; 2],
}

pub struct Field {
    leds: Option<FieldLeds>,
    keyboard: Option<FieldKeyboard>,
    switches: Option<FieldSwitches>,
    gates: Option<FieldGates>,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Led {
    on: u16,
    off: u16,
}

//struct representing an entire, auto incremented, update
#[repr(C, packed)]
struct LedTxBuffer {
    reg: u8,
    leds: [Led; 16],
}

impl Default for Led {
    fn default() -> Self {
        //full off
        Self { on: 0, off: 0x1001 }
    }
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

///PCA9685 Led Driver
pub struct LedDriver {
    addr: u8,
    buffer: LedTxBuffer,
}

//TODO DMA
impl LedDriver {
    ///Initialize the Led driver with the given address.
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

    /// Set all the buffered values for all LEDs to the given brightness.
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

    /// Set the buffered value for the given LED to the given brightness.
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

    /// Update all the leds.
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
impl FieldLeds {
    pub fn new(
        i2cd: hal::stm32::I2C1,
        i2crec: hal::rcc::rec::I2c1,
        scl: hal::gpio::gpiob::PB8<hal::gpio::Analog>,
        sda: hal::gpio::gpiob::PB9<hal::gpio::Analog>,
        clocks: &hal::rcc::CoreClocks,
    ) -> Self {
        let mut i2c = i2cd.i2c(
            (
                scl.into_alternate_af4().set_open_drain(),
                sda.into_alternate_af4().set_open_drain(),
            ),
            1.mhz(),
            i2crec,
            clocks,
        );

        let drivers = [
            LedDriver::new(&mut i2c, LED_ADDR0),
            LedDriver::new(&mut i2c, LED_ADDR1),
        ];
        Self { i2c, drivers }
    }

    pub fn button_set(&mut self, index: usize, brightness: u8) {
        assert!(index < 16);
        let driver = &mut self.drivers[0];
        if index < 8 {
            driver.set(index, brightness);
        } else {
            //the lower row counts backwards
            driver.set(15 - (index - 8), brightness);
        }
    }

    pub fn button_set_all(&mut self, brightness: u8) {
        self.drivers[0].set_all(brightness);
    }

    pub fn pot_set(&mut self, index: usize, brightness: u8) {
        assert!(index < 8);
        self.drivers[1].set(index, brightness);
    }

    pub fn pot_set_all(&mut self, brightness: u8) {
        self.drivers[1].set_all(brightness);
    }

    pub fn draw(&mut self) {
        for driver in self.drivers.iter() {
            driver.draw(&mut self.i2c)
        }
    }
}

impl Field {
    pub fn new(
        //leds
        i2c_dev: hal::stm32::I2C1,
        i2c_rec: hal::rcc::rec::I2c1,
        i2c_scl: hal::gpio::gpiob::PB8<hal::gpio::Analog>,
        i2c_sda: hal::gpio::gpiob::PB9<hal::gpio::Analog>,

        //switches
        sw1: hal::gpio::gpiob::PB15<hal::gpio::Analog>,
        sw2: hal::gpio::gpiob::PB14<hal::gpio::Analog>,

        //keyboard
        keyboard_data: hal::gpio::gpiod::PD11<hal::gpio::Analog>,
        keyboard_latch: hal::gpio::gpiog::PG9<hal::gpio::Analog>,
        keyboard_clock: hal::gpio::gpioa::PA2<hal::gpio::Analog>,

        //gates
        gate_in: hal::gpio::gpiob::PB12<hal::gpio::Analog>,
        gate_out: hal::gpio::gpioc::PC0<hal::gpio::Analog>,

        //oled display
        oled_spi_dev: hal::stm32::SPI1,
        oled_spi_rec: hal::rcc::rec::Spi1,
        oled_nss: hal::gpio::gpiog::PG10<hal::gpio::Analog>,
        oled_sck: hal::gpio::gpiog::PG11<hal::gpio::Analog>,
        oled_cmd: hal::gpio::gpiob::PB4<hal::gpio::Alternate<hal::gpio::AF0>>,
        oled_mosi: hal::gpio::gpiob::PB5<hal::gpio::Analog>,

        //clocks
        delay: &mut hal::delay::Delay,
        clocks: &hal::rcc::CoreClocks,
    ) -> Self {
        let oled_spi: hal::spi::Spi<_, _, u8> = oled_spi_dev.spi(
            (
                oled_sck.into_alternate_af5(),
                hal::spi::NoMiso,
                oled_mosi.into_alternate_af5(),
            ),
            hal::spi::MODE_0,
            3.mhz(),
            oled_spi_rec,
            &clocks,
        );
        let mut disp: ssd1309::mode::GraphicsMode<_> = ssd1309::Builder::new()
            .connect(display_interface_spi::SPIInterface::new(
                oled_spi,
                oled_cmd.into_push_pull_output(),
                oled_nss.into_push_pull_output(),
            ))
            .into();

        let mut reset: ssd1309::builder::NoOutputPin<()> = ssd1309::builder::NoOutputPin::new();
        disp.reset(&mut reset, delay).unwrap();
        disp.init().unwrap();
        disp.flush().unwrap();

        let style: TextStyle<_, Font6x8> = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let text = Text::new("Hello Daisy!", Point::new(0, 0)).into_styled(style);
        text.draw(&mut disp).unwrap();

        let text = Text::new(" - Rust", Point::new(0, 10)).into_styled(style);
        text.draw(&mut disp).unwrap();

        disp.flush().unwrap();

        Self {
            leds: Some(FieldLeds::new(i2c_dev, i2c_rec, i2c_scl, i2c_sda, clocks)),
            keyboard: Some(FieldKeyboard::new(
                keyboard_data,
                keyboard_latch,
                keyboard_clock,
            )),
            switches: Some((sw1.into_pull_up_input(), sw2.into_pull_up_input())),
            gates: Some((
                gate_in.into_floating_input(),
                gate_out.into_push_pull_output(),
            )),
        }
    }

    /// Get a mutable reference to the LEDs
    pub fn leds(&mut self) -> Option<&mut FieldLeds> {
        self.leds.as_mut()
    }

    /// Get the LED struct.
    ///
    /// # Panics
    /// Will panic if done more than once.
    pub fn split_leds(&mut self) -> FieldLeds {
        self.leds.take().unwrap()
    }

    /// Get the keyboard.
    ///
    /// # Panics
    /// Will panic if done more than once.
    pub fn split_keyboard(&mut self) -> FieldKeyboard {
        self.keyboard.take().unwrap()
    }

    /// Get the switches tuple.
    ///
    /// # Panics
    /// Will panic if done more than once.
    pub fn split_switches(&mut self) -> FieldSwitches {
        self.switches.take().unwrap()
    }

    /// Get the gates tuple.
    ///
    /// # Panics
    /// Will panic if done more than once.
    pub fn split_gates(&mut self) -> FieldGates {
        self.gates.take().unwrap()
    }
}

struct FieldShiftDelay;
pub struct FieldKeyboard {
    sreg: ShiftKeyboard,
}

type ShiftKeyboard = ShiftIn<
    hal::gpio::gpiog::PG9<hal::gpio::Output<hal::gpio::PushPull>>,
    hal::gpio::gpioa::PA2<hal::gpio::Output<hal::gpio::PushPull>>,
    hal::gpio::gpiod::PD11<hal::gpio::Input<hal::gpio::Floating>>,
    FieldShiftDelay,
    2,
>;

impl FieldKeyboard {
    pub fn new(
        data: hal::gpio::gpiod::PD11<hal::gpio::Analog>,
        latch: hal::gpio::gpiog::PG9<hal::gpio::Analog>,
        clock: hal::gpio::gpioa::PA2<hal::gpio::Analog>,
    ) -> Self {
        let latch = latch.into_push_pull_output();
        let clock = clock.into_push_pull_output();
        let data = data.into_floating_input();
        let sreg = ShiftIn::new(latch, clock, data, FieldShiftDelay);

        Self { sreg }
    }

    /// Read in all the data
    pub fn read(&mut self) -> [u8; 2] {
        //shift data read in backwards, re-order & invert
        let mut o: [u8; 2] = [0; 2];
        let r = self.sreg.read();
        for i in 0..2 {
            let byte = !r[if i == 0 { 1 } else { 0 }];
            o[i] = 0
                | (byte & (1 << 0)) << 7
                | (byte & (1 << 1)) << 5
                | (byte & (1 << 2)) << 3
                | (byte & (1 << 3)) << 1
                | (byte & (1 << 4)) >> 1
                | (byte & (1 << 5)) >> 3
                | (byte & (1 << 6)) >> 5
                | (byte & (1 << 7)) >> 7;
        }
        o
    }
}

impl ShiftClockDelay for FieldShiftDelay {
    //clock freq max is 3MHz at 5v.. same at 3.3??
    //166ns * 2 -> 332ns -> ~3MHz
    fn delay(&self, _delay: ShiftDelay) {
        crate::delay_ns(166);
    }
}
