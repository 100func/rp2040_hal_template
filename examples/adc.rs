#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// Alias for our HAL crate
use defmt::*;
use defmt_rtt as _;
use hal::Clock;
use panic_probe as _;
use rp2040_hal as hal;

use embedded_hal::adc::OneShot;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[rp2040_hal::entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let core = hal::pac::CorePeripherals::take().unwrap();

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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);

    let mut temperature_sensor = adc.take_temp_sensor().unwrap();
    let mut adc_pin_0 = hal::adc::AdcPin::new(pins.gpio26);
    loop {
        let temp_sens_adc_counts: u16 = adc.read(&mut temperature_sensor).unwrap();
        let pin_adc_counts: u16 = adc.read(&mut adc_pin_0).unwrap();
        info!(
            "Temprature: {}, adc: {}",
            temp_sens_adc_counts, pin_adc_counts
        );
        delay.delay_ms(1000);
    }
}
