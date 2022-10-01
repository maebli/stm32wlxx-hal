// Turns the RGB Led red,green,blue,white,off in a loop
// will work only for TTN Generic node with STM32WL5x
// see https://github.com/TheThingsIndustries/generic-node-se

#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler
use stm32wlxx_hal::{
    self as hal,
    cortex_m::{self, delay::Delay},
    gpio::{pins, Output, PinState, PortB},
    pac,
    util::new_delay,
};

#[hal::cortex_m_rt::entry]
fn main() -> ! {
    let mut dp: pac::Peripherals = defmt::unwrap!(pac::Peripherals::take());
    let cp: pac::CorePeripherals = defmt::unwrap!(pac::CorePeripherals::take());

    let gpiob: PortB = PortB::split(dp.GPIOB, &mut dp.RCC);
    let (mut red_led, mut green_led, mut blue_led): (Output<pins::B5>, Output<pins::B6>, Output<pins::B7>) =
        cortex_m::interrupt::free(|cs| {
            (
                Output::default(gpiob.b5, cs),
                Output::default(gpiob.b6, cs),
                Output::default(gpiob.b7, cs),
            )
        });

    let mut delay: Delay = new_delay(cp.SYST, &dp.RCC);

    defmt::info!("Starting blinky");

    loop {
        for &level in &[PinState::High, PinState::Low] {
            red_led.set_level(level);
            delay.delay_ms(600);
            green_led.set_level(level);
            delay.delay_ms(600);
            blue_led.set_level(level);
            delay.delay_ms(600);
        }
    }
}
