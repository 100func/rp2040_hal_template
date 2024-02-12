#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use hal::gpio;
use hal::gpio::Interrupt::EdgeLow;
use hal::pac;
use hal::pac::interrupt;
use panic_probe as _;
use rp2040_hal as hal;

// bootloader code
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

type GreenLed = gpio::Pin<gpio::bank0::Gpio25, gpio::FunctionSioOutput, gpio::PullDown>;
type Button = gpio::Pin<gpio::bank0::Gpio0, gpio::FunctionSioInput, gpio::PullUp>;
type GreenLedAndButton = (GreenLed, Button);

static GLOBAL_PINS: Mutex<RefCell<Option<GreenLedAndButton>>> = Mutex::new(RefCell::new(None));

#[rp2040_hal::entry]
fn main() -> ! {
    info!("Program start!");
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // Red LED: GPIO23
    let mut red_led = pins.gpio23.into_push_pull_output();

    // Orange LED: GPIO24
    let mut orange_led = pins.gpio24.into_push_pull_output();

    // Green LED: GPIO25
    let green_led = pins.gpio25.into_push_pull_output();

    // Button: GPIO0
    let button = pins.gpio0.reconfigure();

    button.set_interrupt_enabled(EdgeLow, true);

    critical_section::with(|cs| {
        GLOBAL_PINS.borrow(cs).replace(Some((green_led, button)));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
    }

    loop {
        red_led.set_high().unwrap();
        timer.delay_ms(500);
        red_led.set_low().unwrap();
        timer.delay_ms(500);

        orange_led.set_high().unwrap();
        timer.delay_ms(500);
        orange_led.set_low().unwrap();
        timer.delay_ms(500);
    }
}

#[interrupt]
fn IO_IRQ_BANK0() {
    static mut REDLED_AND_BUTTON: Option<GreenLedAndButton> = None;
    static mut CNT: u32 = 0;

    if REDLED_AND_BUTTON.is_none() {
        critical_section::with(|cs| {
            *REDLED_AND_BUTTON = GLOBAL_PINS.borrow(cs).take();
        })
    }

    if let Some(gpios) = REDLED_AND_BUTTON {
        let (green_led, button) = gpios;
        if button.interrupt_status(EdgeLow) {
            green_led.set_high().unwrap();
            while button.is_low().unwrap() {
                *CNT += 1;
                info!("button start");
            }
            info!("cnt:{}", CNT);
            info!("button end");
            *CNT = 0;
            green_led.set_low().unwrap();

            button.clear_interrupt(EdgeLow);
        }
    }
}
