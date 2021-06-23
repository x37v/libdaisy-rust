//! IS25LP064A: 64Mbit/8Mbyte flash memory
//!
//! https://www.issi.com/WW/pdf/25LP032-64A-B.pdf
//!

// 133Mhz
// bank 1
//  pin_group[DSY_QSPI_PIN_IO0] = dsy_pin(DSY_GPIOF, 8);
//  pin_group[DSY_QSPI_PIN_IO1] = dsy_pin(DSY_GPIOF, 9);
//  pin_group[DSY_QSPI_PIN_IO2] = dsy_pin(DSY_GPIOF, 7);
//  pin_group[DSY_QSPI_PIN_IO3] = dsy_pin(DSY_GPIOF, 6);
//  pin_group[DSY_QSPI_PIN_CLK] = dsy_pin(DSY_GPIOF, 10);
//  pin_group[DSY_QSPI_PIN_NCS] = dsy_pin(DSY_GPIOG, 6);

use stm32h7xx_hal::{
    gpio::{gpiof, gpiog, Analog, OpenDrain, Output},
    hal::digital::v2::OutputPin,
    prelude::*,
    rcc,
    xspi::{Config, QspiMode, QspiWord},
};

struct Flash {
    qspi: stm32h7xx_hal::xspi::Qspi<stm32h7xx_hal::stm32::QUADSPI>,
    ncs: gpiog::PG6<Output<OpenDrain>>,
}

impl Flash {
    /// Initialize the flash quad spi interface
    pub fn new(
        regs: stm32h7xx_hal::device::QUADSPI,
        prec: rcc::rec::Qspi,
        clocks: &rcc::CoreClocks,
        pf6: gpiof::PF6<Analog>,
        pf7: gpiof::PF7<Analog>,
        pf8: gpiof::PF8<Analog>,
        pf9: gpiof::PF9<Analog>,
        pf10: gpiof::PF10<Analog>,
        pg6: gpiog::PG6<Analog>,
    ) -> Self {
        let mut ncs = pg6.into_open_drain_output();
        ncs.set_high().unwrap();

        let sck = pf10.into_alternate_af9();
        let io0 = pf8.into_alternate_af10();
        let io1 = pf9.into_alternate_af10();
        let io2 = pf7.into_alternate_af9();
        let io3 = pf6.into_alternate_af9();

        let config = Config::new(133.mhz()).mode(QspiMode::OneBit);
        let mut qspi = regs.bank1((sck, io0, io1, io2, io3), config, &clocks, prec);

        //setup in qspi mode
        ncs.set_low().unwrap();
        qspi.write_extended(QspiWord::U8(0x35), QspiWord::None, QspiWord::None, &[])
            .unwrap();
        while qspi.is_busy().is_err() {}
        ncs.set_high().unwrap();
        qspi.configure_mode(QspiMode::FourBit).unwrap();

        Flash { qspi, ncs }
    }
}

/// Initialize the flash quad spi interface
pub fn init(
    regs: stm32h7xx_hal::device::QUADSPI,
    prec: rcc::rec::Qspi,
    clocks: &rcc::CoreClocks,
    pf6: gpiof::PF6<Analog>,
    pf7: gpiof::PF7<Analog>,
    pf8: gpiof::PF8<Analog>,
    pf9: gpiof::PF9<Analog>,
    pf10: gpiof::PF10<Analog>,
    pg6: gpiog::PG6<Analog>,
) -> stm32h7xx_hal::xspi::Qspi<stm32h7xx_hal::stm32::QUADSPI> {
    let mut cs = pg6.into_open_drain_output();

    let sck = pf10.into_alternate_af9();
    let io0 = pf8.into_alternate_af10();
    let io1 = pf9.into_alternate_af10();
    let io2 = pf7.into_alternate_af9();
    let io3 = pf6.into_alternate_af9();

    let config = Config::new(133.mhz()).mode(QspiMode::OneBit);
    let mut qspi = regs.bank1((sck, io0, io1, io2, io3), config, &clocks, prec);
    cs.set_high().unwrap();
    qspi.write_extended(QspiWord::U8(0x35), QspiWord::None, QspiWord::None, &[])
        .unwrap();
    while qspi.is_busy().is_err() {}
    cs.set_low().unwrap();
    qspi.configure_mode(QspiMode::FourBit).unwrap();
    qspi
}
