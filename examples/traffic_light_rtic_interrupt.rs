// traffic_light_button_rtic.rs
#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use panic_probe as _;

// bootloader code
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rtic::app(
    device = hal::pac,
    dispatchers = [TIMER_IRQ_1],
)]
mod app {
    use defmt::info;
    use embedded_hal::digital::v2::OutputPin;
    use rp2040_hal as hal;
    use rtic_monotonics::rp2040::*;

    use crate::XTAL_FREQ_HZ;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        green_led: hal::gpio::Pin<
            hal::gpio::bank0::Gpio22,
            hal::gpio::FunctionSioOutput,
            hal::gpio::PullDown,
        >,
        orange_led: hal::gpio::Pin<
            hal::gpio::bank0::Gpio21,
            hal::gpio::FunctionSioOutput,
            hal::gpio::PullDown,
        >,
        red_led: hal::gpio::Pin<
            hal::gpio::bank0::Gpio20,
            hal::gpio::FunctionSioOutput,
            hal::gpio::PullDown,
        >,
        button: hal::gpio::Pin<
            hal::gpio::bank0::Gpio23,
            hal::gpio::FunctionSioInput,
            hal::gpio::PullUp,
        >,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        let timer_token = rtic_monotonics::create_rp2040_monotonic_token!();
        Timer::start(ctx.device.TIMER, &mut ctx.device.RESETS, timer_token);
        let mut watchdog = hal::Watchdog::new(ctx.device.WATCHDOG);

        let _clocks = hal::clocks::init_clocks_and_plls(
            XTAL_FREQ_HZ,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut ctx.device.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = hal::Sio::new(ctx.device.SIO);
        let pins = hal::gpio::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut ctx.device.RESETS,
        );
        let green_led = pins.gpio22.into_push_pull_output();
        let orange_led = pins.gpio21.into_push_pull_output();
        let mut red_led = pins.gpio20.into_push_pull_output();

        let button = pins.gpio23.into_pull_up_input();

        button.set_interrupt_enabled(hal::gpio::Interrupt::EdgeLow, true);
        red_led.set_high().unwrap();

        (
            Shared {},
            Local {
                green_led,
                orange_led,
                red_led,
                button,
            },
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(local = [green_led, orange_led, red_led], priority = 1)]
    async fn change_lights(ctx: change_lights::Context) {
        info!("Changing lights");
        let green_led = ctx.local.green_led;
        let orange_led = ctx.local.orange_led;
        let red_led = ctx.local.red_led;

        red_led.set_low().unwrap();

        green_led.set_high().unwrap();
        Timer::delay(2000.millis()).await;
        green_led.set_low().unwrap();

        for _ in 1..4 {
            orange_led.set_high().unwrap();
            Timer::delay(500.millis()).await;
            orange_led.set_low().unwrap();
            Timer::delay(500.millis()).await;
        }
        orange_led.set_low().unwrap();
        red_led.set_high().unwrap();
    }

    #[task(binds = IO_IRQ_BANK0, local = [button])]
    fn interrupt_button(ctx: interrupt_button::Context) {
        info!("Button interrupt");
        let button = ctx.local.button;
        if button.interrupt_status(hal::gpio::Interrupt::EdgeLow) {
            change_lights::spawn().unwrap();
            button.clear_interrupt(hal::gpio::Interrupt::EdgeLow)
        }
    }
}
