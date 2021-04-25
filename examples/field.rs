//! examples/field.rs
#![no_main]
#![no_std]
use log::info;

use libdaisy::{
    field::{Field, FieldKeyboard, FieldLeds},
    gpio, logger,
    prelude::*,
    system::System,
};
use stm32h7xx_hal::{
    block, stm32,
    timer::{Event, Timer},
};

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: gpio::SeedLed,
        field_leds: FieldLeds,
        keyboard: FieldKeyboard,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();

        let device = ctx.device;
        let mut ccdr = System::init_clocks(device.PWR, device.RCC, &device.SYSCFG);
        let mut timer2 = device
            .TIM2
            .timer(100.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        timer2.listen(Event::TimeOut);

        info!("Startup done!");

        timer2.set_freq(50.ms());

        let gpioa = device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpiog = device.GPIOG.split(ccdr.peripheral.GPIOG);

        let mut gpio = crate::gpio::GPIO::init(
            gpioc.pc7,
            gpiob.pb11,
            Some(gpiob.pb12),
            Some(gpioc.pc11),
            Some(gpioc.pc10),
            Some(gpioc.pc9),
            Some(gpioc.pc8),
            Some(gpiod.pd2),
            Some(gpioc.pc12),
            Some(gpiog.pg10),
            Some(gpiog.pg11),
            Some(gpiob.pb4),
            Some(gpiob.pb5),
            Some(gpiob.pb8),
            Some(gpiob.pb9),
            Some(gpiob.pb6),
            Some(gpiob.pb7),
            Some(gpioc.pc0),
            Some(gpioa.pa3),
            Some(gpiob.pb1),
            Some(gpioa.pa7),
            Some(gpioa.pa6),
            Some(gpioc.pc1),
            Some(gpioc.pc4),
            Some(gpioa.pa5),
            Some(gpioa.pa4),
            Some(gpioa.pa1),
            Some(gpioa.pa0),
            Some(gpiod.pd11),
            Some(gpiog.pg9),
            Some(gpioa.pa2),
            Some(gpiob.pb14),
            Some(gpiob.pb15),
        );
        let mut field = Field::new(
            device.I2C1,
            ccdr.peripheral.I2C1,
            gpio.daisy11.take().unwrap(),
            gpio.daisy12.take().unwrap(),
            &mut ccdr.clocks,
        );

        let mut leds = field.split_leds();

        let keyboard = FieldKeyboard::new(
            gpio.daisy26.take().unwrap(),
            gpio.daisy27.take().unwrap(),
            gpio.daisy28.take().unwrap(),
        );

        init::LateResources {
            seed_led: gpio.led,
            timer2,
            keyboard,
            field_leds: leds,
        }
    }

    #[task( binds = TIM2, resources = [timer2, seed_led, field_leds, keyboard] )]
    fn blink(ctx: blink::Context) {
        static mut LED_IS_ON: bool = true;
        static mut BRIGHTNESS: u8 = 0;

        ctx.resources.timer2.clear_irq();

        let r = ctx.resources.keyboard.read();

        for by in 0..2 {
            let byte = r[by];
            for b in 0..8 {
                ctx.resources
                    .field_leds
                    .button_set(by * 8 + b, if byte & (1 << b) != 0 { 0xFF } else { 0 });
            }
        }

        ctx.resources.field_leds.pot_set_all(*BRIGHTNESS);
        if *LED_IS_ON {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }
        *LED_IS_ON = !(*LED_IS_ON);
        ctx.resources.field_leds.draw();

        *BRIGHTNESS = (*BRIGHTNESS).wrapping_add(16);
    }
};
