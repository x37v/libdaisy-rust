//! examples/blinky.rs
#![no_main]
#![no_std]
use log::info;
// Includes a panic handler and optional logging facilities
use libdaisy::logger;

use stm32h7xx_hal::stm32;
use stm32h7xx_hal::timer::Timer;

use libdaisy::gpio;
use libdaisy::prelude::*;
use libdaisy::{flash::FlashErase, system};

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
        flash.erase(FlashErase::Sector4K(0)).unwrap();

        //read, should be all 0xFF
        let mut r = [0x0; 32];
        flash.read(0, &mut r).unwrap();
        assert_eq!(r[0], 0xFF);
        assert_eq!(r[31], 0xFF);

        //can program over 1s
        flash.program(0, &[0x1, 0xFF]).unwrap();
        flash.program(1, &[0x2, 0x3]).unwrap();

        //read the new values
        flash.read(0, &mut r).unwrap();
        assert_eq!(r[0], 0x1);
        assert_eq!(r[1], 0x2);
        assert_eq!(r[2], 0x3);

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
