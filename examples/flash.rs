//! examples/blinky.rs
#![no_main]
#![no_std]
use log::info;

use libdaisy::{flash::FlashErase, gpio, logger, prelude::*, system};
use stm32h7xx_hal::{nb, stm32, timer::Timer};

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: gpio::SeedLed,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);
        info!("Startup done!");

        system.timer2.set_freq(500.ms());

        let mut flash = system.flash;

        //takes some time
        nb::block!(flash.erase(FlashErase::Sector4K(0))).unwrap();
        nb::block!(flash.erase(FlashErase::Sector4K(4096))).unwrap();

        //read, should be all 0xFF
        let mut r = [0x0; 64];
        flash.read(0, &mut r).unwrap();
        assert_eq!(r[0], 0xFF);
        assert_eq!(r[31], 0xFF);

        nb::block!(flash.program(0, &[0x42])).unwrap();
        //can program over 1s
        nb::block!(flash.program(32, &[0x1, 0xFF])).unwrap();
        nb::block!(flash.program(33, &[0x2, 0x3])).unwrap();

        //read the new values
        flash.read(0, &mut r).unwrap();
        assert_eq!(r[0], 0x42);
        assert_eq!(r[32], 0x1);
        assert_eq!(r[33], 0x2);
        assert_eq!(r[34], 0x3);

        r[35] = 0x91;
        nb::block!(flash.program(0, &r)).unwrap();
        flash.read(0, &mut r).unwrap();
        assert_eq!(r[35], 0x91);

        flash.read(4096, &mut r).unwrap();
        assert_eq!(r[0], 0xFF);
        r[0] = 0;
        nb::block!(flash.program(4096, &r)).unwrap();
        flash.read(4096, &mut r).unwrap();
        assert_eq!(r[0], 0x00);

        nb::block!(flash.erase(FlashErase::Sector4K(4096))).unwrap();
        flash.read(4096, &mut r).unwrap();
        assert_eq!(r[0], 0xFF);
        flash.read(0, &mut r).unwrap();
        assert_eq!(r[35], 0x91);

        init::LateResources {
            seed_led: system.gpio.led,
            timer2: system.timer2,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task( binds = TIM2, resources = [timer2, seed_led] )]
    fn blink(ctx: blink::Context) {
        static mut LED_IS_ON: bool = true;

        ctx.resources.timer2.clear_irq();

        if *LED_IS_ON {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }
        *LED_IS_ON = !(*LED_IS_ON);
    }
};
