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
    gpio::{gpiof, gpiog, Analog, Speed},
    prelude::*,
    rcc,
    xspi::{Config, QspiMode, QspiWord},
};

pub struct Flash {
    qspi: stm32h7xx_hal::xspi::Qspi<stm32h7xx_hal::stm32::QUADSPI>,
}

/*
 *
 * 6.1 STATUS REGISTER
 * Status Register Format and Status Register Bit Definitionsare described in Table 6.1 & Table 6.2.
 * 0: WIP write in progress(0 == read, 1 == busy)
 * 1: WEL write enable (1 == enabled)
 * 2-5: BP block protection (0 indicates not protected [default])
 * 6: Quad enable, quad output function enable, (1 = enable)
 * 7: Status register write disable (1 == write protected [0 = default])
 *
 * 6.3 READ REGISTER
 *
 *
 * 8.11 SECTOR ERASE OPERATION (SER, D7h/20h)
 *  * instruction, 3 byte address
 *  * WEL is reset after
 * 8.12 BLOCK ERASE OPERATION (BER32K:52h, BER64K:D8h)
 *  * instruction, 3 byte address
 *  * WEL is reset after
 * 8.13 CHIP ERASE OPERATION (CER, C7h/60h)
 *  * instruction only
 *  * WEL is reset after
 * 8.14 WRITE ENABLE OPERATION (WREN, 06h)
 *  * instruction only
 *  * sets WEL
 * 8.16 READ STATUS REGISTER OPERATION (RDSR, 05h)
 *  * instruction, 1 byte read
*/

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
        //let rcc = unsafe { &*stm32h7xx_hal::stm32::RCC::ptr() };
        //rcc.ahb3enr.modify(|_, w| w.qspien().set_bit());

        let _ncs = pg6.into_alternate_af10(); //QUADSPI_BK1_NCS

        let sck = pf10.into_alternate_af9();
        let io0 = pf8.into_alternate_af10();
        let io1 = pf9.into_alternate_af10();
        let io2 = pf7.into_alternate_af9();
        let io3 = pf6.into_alternate_af9();

        let config = Config::new(1.mhz()).mode(QspiMode::OneBit);
        let mut qspi = regs.bank1((sck, io0, io1, io2, io3), config, &clocks, prec);

        /*
        //read info
        let mut info: [u8; 3] = [0xFF; 3];
        while qspi.is_busy().is_err() {}
        qspi.read(0x9F, &mut info).unwrap();

        //read status
        let mut status: [u8; 1] = [0xFF];
        while qspi.is_busy().is_err() {}
        qspi.read(0x05, &mut status).unwrap();

        let mut funct: [u8; 1] = [0xFF];
        while qspi.is_busy().is_err() {}
        qspi.read(0x48, &mut funct).unwrap();
        */

        /*
        //enable write

        //setup in qspi mode
        qspi.write_extended(QspiWord::U8(0x35), QspiWord::None, QspiWord::None, &[])
            .unwrap();
        qspi.configure_mode(QspiMode::FourBit).unwrap();
        */

        //read info
        let mut info: [u8; 3] = [0; 3];
        while qspi.is_busy().is_err() {}
        qspi.read_extended(
            QspiWord::U8(0x9F),
            QspiWord::None,
            QspiWord::None,
            0,
            &mut info,
        )
        .unwrap();

        while qspi.is_busy().is_err() {}
        qspi.write_extended(QspiWord::U8(0x06), QspiWord::None, QspiWord::None, &[])
            .unwrap();

        //read status
        let mut status: [u8; 1] = [0xFF];
        while qspi.is_busy().is_err() {}
        qspi.read_extended(
            QspiWord::U8(0x05),
            QspiWord::None,
            QspiWord::None,
            0,
            &mut status,
        )
        .unwrap();

        while qspi.is_busy().is_err() {}
        Flash { qspi }
    }
}
