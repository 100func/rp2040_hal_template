#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// Alias for our HAL crate
use defmt_rtt as _;
use panic_probe as _;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[rtic::app(
    device = hal::pac,
    dispatchers = [TIMER_IRQ_1]
)]
mod app {
    use defmt::*;
    use defmt_rtt as _;
    use embedded_hal::digital::v2::{InputPin, OutputPin};
    use hal::gpio;
    use panic_probe as _;
    use rp2040_hal as hal;
    use rtic_monotonics::rp2040::*;

    type RedLed = gpio::Pin<gpio::bank0::Gpio23, gpio::FunctionSioOutput, gpio::PullDown>;
    type OrangeLed = gpio::Pin<gpio::bank0::Gpio24, gpio::FunctionSioOutput, gpio::PullDown>;
    type GreenLed = gpio::Pin<gpio::bank0::Gpio25, gpio::FunctionSioOutput, gpio::PullDown>;
    type Button = gpio::Pin<gpio::bank0::Gpio0, gpio::FunctionSioInput, gpio::PullUp>;

    const XTAL_FREQ_HZ: u32 = 12_000_000u32;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        red_led: RedLed,
        orange_led: OrangeLed,
        green_led: GreenLed,
        button: Button,
    }

    #[init(local=[])]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        info!("Program start!");
        let rp2040_timer_token = rtic_monotonics::create_rp2040_monotonic_token!();
        Timer::start(ctx.device.TIMER, &mut ctx.device.RESETS, rp2040_timer_token);

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
        let red_led = pins.gpio23.into_push_pull_output();
        let orange_led = pins.gpio24.into_push_pull_output();
        let green_led = pins.gpio25.into_push_pull_output();

        let button = pins.gpio0.into_pull_up_input();
        button.set_interrupt_enabled(hal::gpio::Interrupt::EdgeLow, true);

        switch_blink::spawn().ok();
        green_lamp::spawn().ok();

        (
            Shared {},
            Local {
                red_led,
                orange_led,
                green_led,
                button,
            },
        )
    }

    #[task(local = [red_led, orange_led])]
    async fn switch_blink(ctx: switch_blink::Context) {
        let red_led = ctx.local.red_led;
        let orange_led = ctx.local.orange_led;

        loop {
            red_led.set_high().unwrap();
            Timer::delay(500.millis()).await;
            red_led.set_low().unwrap();
            Timer::delay(500.millis()).await;

            orange_led.set_high().unwrap();
            Timer::delay(500.millis()).await;
            orange_led.set_low().unwrap();
            Timer::delay(500.millis()).await;
        }
    }

    #[task(local = [green_led, button])]
    async fn green_lamp(ctx: green_lamp::Context) {
        let mut cnt = 0;
        let green_led = ctx.local.green_led;
        let button = ctx.local.button;
        loop {
            if button.is_low().unwrap() {
                if cnt == 0 {
                    info!("button start");
                }
                cnt += 1;
                green_led.set_high().unwrap();
            } else {
                if cnt != 0 {
                    info!("cnt:{}", cnt);
                    info!("button end");
                    cnt = 0;
                }
                green_led.set_low().unwrap();
            }
            Timer::delay(1.millis()).await;
        }
    }
}
