//! examples/field.rs
#![no_main]
#![no_std]

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
)]
mod app {
    use log::info;

    use embedded_graphics::{
        mono_font::{ascii::FONT_5X8 as Font, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        prelude::*,
        primitives::{Circle, PrimitiveStyle},
        text::{Baseline, Text},
    };

    use libdaisy::{
        field::{Field, FieldKeyboard, FieldLeds, FieldSwitches, FIELD_DISPLAY_SIZE},
        gpio, logger,
        prelude::*,
        system::System,
    };
    use stm32h7xx_hal::{
        delay::Delay,
        hal::digital::v2::InputPin,
        stm32,
        timer::{Event, Timer},
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        seed_led: gpio::SeedLed,
        field_leds: FieldLeds,
        keyboard: FieldKeyboard,
        switches: FieldSwitches,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
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

        let mut gpio = gpio::GPIO::init(
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

        let mut delay = Delay::new(ctx.core.SYST, ccdr.clocks);

        let mut field = Field::new(
            //leds
            device.I2C1,
            ccdr.peripheral.I2C1,
            gpio.daisy11.take().unwrap(),
            gpio.daisy12.take().unwrap(),
            //switches
            gpio.daisy30.take().unwrap(),
            gpio.daisy29.take().unwrap(),
            //keyboard
            gpio.daisy26.take().unwrap(),
            gpio.daisy27.take().unwrap(),
            gpio.daisy28.take().unwrap(),
            //gates
            gpio.daisy0.take().unwrap(),
            gpio.daisy15.take().unwrap(),
            //midi
            gpio.daisy13.take().unwrap(),
            gpio.daisy14.take().unwrap(),
            device.USART1,
            ccdr.peripheral.USART1,
            //oled display
            device.SPI1,
            ccdr.peripheral.SPI1,
            gpio.daisy7.take().unwrap(),
            gpio.daisy8.take().unwrap(),
            gpio.daisy9.take().unwrap(),
            gpio.daisy10.take().unwrap(),
            //clocks
            &mut delay,
            &mut ccdr.clocks,
        );

        let leds = field.split_leds();
        let keyboard = field.split_keyboard();
        let switches = field.split_switches();

        let mut disp = field.split_display();

        let style = MonoTextStyleBuilder::new()
            .font(&Font)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let text = Text::with_baseline("Hello Daisy!", Point::new(0, 0), style, Baseline::Top);
        text.draw(&mut disp).unwrap();

        let text = Text::with_baseline(" - Rust", Point::new(0, 10), style, Baseline::Top);
        text.draw(&mut disp).unwrap();

        let w = 8;
        let c = Circle::new(
            Point::new(
                ((FIELD_DISPLAY_SIZE.0 - w) / 2) as _,
                ((FIELD_DISPLAY_SIZE.1 - w) / 2) as _,
            ),
            w as u32,
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
        c.draw(&mut disp).unwrap();

        disp.flush().unwrap();

        (
            Shared {},
            Local {
                seed_led: gpio.led,
                timer2,
                keyboard,
                switches,
                field_leds: leds,
            },
            init::Monotonics(),
        )
    }

    #[task(
        binds = TIM2,
        local = [
        timer2, seed_led, field_leds, keyboard, switches,
        led_is_on: bool = true, brightness: u8 = 0
        ]
    )]
    fn blink(ctx: blink::Context) {
        ctx.local.timer2.clear_irq();

        let r = ctx.local.keyboard.read();

        for by in 0..2 {
            let byte = r[by];
            for b in 0..8 {
                ctx.local
                    .field_leds
                    .button_set(by * 8 + b, if byte & (1 << b) != 0 { 0xFF } else { 0 });
            }
        }

        ctx.local.field_leds.pot_set_all(*ctx.local.brightness);
        if *ctx.local.led_is_on {
            ctx.local.seed_led.set_high().unwrap();
        } else {
            ctx.local.seed_led.set_low().unwrap();
        }
        *ctx.local.led_is_on = !(*ctx.local.led_is_on);
        ctx.local.field_leds.draw();

        match ctx.local.switches.0.is_low() {
            Ok(true) => {
                *ctx.local.brightness = (*ctx.local.brightness).wrapping_add(16);
            }
            _ => (),
        };
        match ctx.local.switches.1.is_low() {
            Ok(true) => {
                *ctx.local.brightness = (*ctx.local.brightness).wrapping_add(64);
            }
            _ => (),
        };
    }
}
