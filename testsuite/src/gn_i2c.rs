#![no_std]
#![no_main]

use defmt::unwrap;
use defmt_rtt as _; // global logger
use nucleo_wl55jc_bsp::hal::{
    cortex_m::{self, delay::Delay},
    embedded_hal::blocking::i2c::{WriteRead,Write},
    gpio::{pins, Output, PinState,PortB,PortA},
    i2c::{I2c1},
    pac::{self},
    rcc,
    util::new_delay,
};
use panic_probe as _;

const I2C_FREQUENCY: u32 = 100_000;

#[defmt_test::tests]
mod tests {
    use super::*;

    #[init]
    fn init() -> I2c1<(pins::A9, pins::A10)> {
        cortex_m::interrupt::free(|cs| {
            let mut dp: pac::Peripherals = unwrap!(pac::Peripherals::take());
            let cp: pac::CorePeripherals = defmt::unwrap!(pac::CorePeripherals::take());

            unsafe { rcc::set_sysclk_msi_max(&mut dp.FLASH, &mut dp.PWR, &mut dp.RCC, cs) };
            let gpioa: PortA = PortA::split(dp.GPIOA, &mut dp.RCC);
            let gpiob: PortB = PortB::split(dp.GPIOB, &mut dp.RCC);
 
            let mut sensor_power_enable: Output<pins::B12> =
                cortex_m::interrupt::free(|cs| Output::default(gpiob.b12, cs));


            // Power the sensor
            sensor_power_enable.set_level(PinState::High);
            let mut delay: Delay = new_delay(cp.SYST, &dp.RCC);
            // wait for the sensor to be powered up

            delay.delay_us(240);

            I2c1::new(
                dp.I2C1,
                (gpioa.a9, gpioa.a10),
                I2C_FREQUENCY,
                &mut dp.RCC,
                false,
                cs,
            )
        })
    }


    #[test]
    fn shtc3_measurement(i2c: &mut I2c1<(pins::A9, pins::A10)>) {
        defmt::warn!("A SHTC3 sensor must be connected to the board on pins A9 (SCL) & A10 (SDA) for this test to work");
        let mut cmd: [u8; 2] = [0x35, 0x17];
        let mut response: [u8; 3] = [0; 3];

        let result = i2c.write(0x70, &cmd);

        match result {
            Ok(()) => defmt::info!("SHTC3 was woken up"),
            Err(e) => defmt::error!("I2C error: {}", e),
        }

        cmd = [0xEF, 0xC8];

        let result = i2c.write_read(0x70, &cmd, &mut response);

        match result {
            Ok(()) => defmt::info!("SHTC3 ID = {:x} ",response),
            Err(e) => defmt::error!("I2C error: {}", e),
        }
    }

}

