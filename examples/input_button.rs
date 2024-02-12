#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use hal::pac;
use panic_probe as _;
use rp2040_hal as hal;

// bootloader code
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rp2040_hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let _clocks = hal::clocks::init_clocks_and_plls(
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

    // Red LED: GPIO23
    let mut red_led = pins.gpio23.into_push_pull_output();

    // Button: GPIO0
    let button = pins.gpio0.into_pull_up_input();

    let mut cnt = 0;

    loop {
        if button.is_low().unwrap() {
            if cnt == 0 {
                info!("button start");
            }
            cnt += 1;
            red_led.set_high().unwrap();
        } else {
            if cnt != 0 {
                info!("cnt:{}", cnt);
                info!("button end");
                cnt = 0;
            }
            red_led.set_low().unwrap();
        }
    }
}
