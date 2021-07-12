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
    xspi::{Config, QspiError, QspiMode, QspiWord},
};

pub type FlashResult<T> = Result<T, QspiError>;

/// Flash erasure enum
#[derive(Clone, Copy)]
pub enum FlashErase {
    ///The whole chip
    Chip,
    ///4Kbyte sector
    Sector4K(u16),
    ///32Kbyte block
    Block32K(u8),
    ///64Kbyte block
    Block64K(u8),
}

/// Flash memory peripheral
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
    fn wait(&mut self) {
        while self.qspi.is_busy().is_err() {}
    }

    fn wait_write(&mut self) -> FlashResult<()> {
        loop {
            match self.status() {
                Ok(status) => {
                    if status & 0x01 == 0 {
                        return Ok(());
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn write_command(&mut self, cmd: u8) -> FlashResult<()> {
        self.wait();
        self.qspi
            .write_extended(QspiWord::U8(cmd), QspiWord::None, QspiWord::None, &[])
    }

    fn write_reg(&mut self, cmd: u8, data: u8) -> FlashResult<()> {
        self.wait();
        self.qspi
            .write_extended(QspiWord::U8(cmd), QspiWord::None, QspiWord::None, &[data])
    }

    fn enable_write(&mut self) -> FlashResult<()> {
        self.write_command(0x06)
    }

    fn assert_info(&mut self) {
        let mut info: [u8; 3] = [0; 3];
        self.wait();
        self.qspi
            .read_extended(
                QspiWord::U8(0x9F),
                QspiWord::None,
                QspiWord::None,
                0,
                &mut info,
            )
            .unwrap();
        assert_eq!(&info, &[157, 96, 23]);
    }

    fn status(&mut self) -> FlashResult<u8> {
        let mut status: [u8; 1] = [0xFF];
        self.wait();
        self.qspi
            .read_extended(
                QspiWord::U8(0x05),
                QspiWord::None,
                QspiWord::None,
                0,
                &mut status,
            )
            .map(|_| status[0])
    }

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
        let _ncs = pg6.into_alternate_af10().set_speed(Speed::VeryHigh); //QUADSPI_BK1_NCS

        let sck = pf10.into_alternate_af9().set_speed(Speed::VeryHigh);
        let io0 = pf8.into_alternate_af10().set_speed(Speed::VeryHigh);
        let io1 = pf9.into_alternate_af10().set_speed(Speed::VeryHigh);
        let io2 = pf7.into_alternate_af9().set_speed(Speed::VeryHigh);
        let io3 = pf6.into_alternate_af9().set_speed(Speed::VeryHigh);

        let config = Config::new(133.mhz()).mode(QspiMode::OneBit);
        let qspi = regs.bank1((sck, io0, io1, io2, io3), config, &clocks, prec);

        let mut flash = Flash { qspi };

        //enable quad
        flash.enable_write().unwrap();
        flash.write_command(0x35).unwrap();
        flash.qspi.configure_mode(QspiMode::FourBit).unwrap();

        flash.enable_write().unwrap();
        //only enable write, nothing else
        flash.write_reg(0x01, 0b0000_0010).unwrap();
        flash.wait_write().unwrap();
        flash.assert_info();

        //setup read parameters, no wrap, default strength, default burst, 8 dummy cycles
        //pg 19
        flash.enable_write().unwrap();
        flash.write_reg(0xC0, 0b1111_1000).unwrap();
        flash.wait_write().unwrap();

        flash
    }

    /// Erase all or some of the chip.
    ///
    /// Remarks:
    /// - Erasing sets all the bits in the given area to `1`.
    /// - The memory array of the IS25LP064A/032A is organized into uniform 4 Kbyte sectors or
    /// 32/64 Kbyte uniform blocks (a block consists of eight/sixteen adjacent sectors
    /// respectively).
    pub fn erase(&mut self, op: FlashErase) -> FlashResult<()> {
        self.enable_write()?;
        self.wait();
        match op {
            FlashErase::Chip => self.write_command(0x60),
            FlashErase::Sector4K(s) => {
                assert!(s <= 2047);
                self.qspi.write_extended(
                    QspiWord::U8(0xD7),
                    QspiWord::U24(s as _),
                    QspiWord::None,
                    &[],
                )
            }
            FlashErase::Block32K(b) => self.qspi.write_extended(
                QspiWord::U8(0x52),
                QspiWord::U24(b as _),
                QspiWord::None,
                &[],
            ),
            FlashErase::Block64K(b) => {
                assert!(b <= 127);
                self.qspi.write_extended(
                    QspiWord::U8(0xD8),
                    QspiWord::U24(b as _),
                    QspiWord::None,
                    &[],
                )
            }
        }?;
        self.wait_write()
    }

    /// Read `data` out of the flash starting at the given `address`
    pub fn read(&mut self, address: u32, data: &mut [u8]) -> FlashResult<()> {
        let mut addr = address;
        //see page 34 for allowing to skip instruction
        assert!((addr as usize + data.len()) < 0x800000);
        for chunk in data.chunks_mut(32) {
            self.wait();
            self.qspi.read_extended(
                QspiWord::U8(0xEB),
                QspiWord::U24(addr),
                QspiWord::U8(0x00), //only A in top byte does anything
                8,
                chunk,
            )?;
            addr += 32;
        }
        Ok(())
    }

    /// Program `data` into the flash starting at the given `address`
    ///
    /// Remarks:
    /// - This operation can only set 1s to 0s, you must use `erase` to set a 0 to a 1.
    /// - The starting byte can be anywhere within the page (256 byte chunk). When the end of the
    /// page is reached, the address will wrap around to the beginning of the same page. If the
    /// data to be programmed are less than a full page, the data of all other bytes on the same
    /// page will remain unchanged.
    pub fn program(&mut self, address: u32, data: &[u8]) -> FlashResult<()> {
        let mut addr = address;
        assert!((addr as usize + data.len()) < 0x800000);
        for chunk in data.chunks(32) {
            self.enable_write()?;
            self.wait();
            self.qspi.write_extended(
                QspiWord::U8(0x02),
                QspiWord::U24(addr),
                QspiWord::None,
                chunk,
            )?;
            self.wait_write()?;
            addr += 32;
        }
        Ok(())
    }
}
