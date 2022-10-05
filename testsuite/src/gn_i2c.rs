#![no_std]
#![no_main]

use defmt::unwrap;
use defmt_rtt as _; // global logger
use nucleo_wl55jc_bsp::hal::{
    cortex_m::{self, delay::Delay},
    embedded_hal::blocking::i2c::{WriteRead,Write,Read},
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

    struct TestArgs {
        i2c: I2c1<(pins::A9, pins::A10)>,
        delay: Delay,
    }

    #[init]
    fn init() -> TestArgs {
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

            // wait the max time for the sensors to be powered up,
            // 5 ms for the accelerometer
            // 240us for temperature and humidity sensor
            let mut delay: Delay = new_delay(cp.SYST, &dp.RCC);
            delay.delay_ms(5);

            let i2c = I2c1::new(
                dp.I2C1,
                (gpioa.a9, gpioa.a10),
                I2C_FREQUENCY,
                &mut dp.RCC,
                true,
                cs,
                );

            TestArgs{
                i2c,
                delay
            }

        })
    }



    #[test]
    fn lis2dh12_measurement(ta: &mut TestArgs) {

        defmt::warn!("A LIS2DH12 sensor must be connected to the board on pins A9 (SCL) & A10 (SDA) for this test to work");

        let mut response =[0u8];

        // Read the WHO_AM_I register
        let result = ta.i2c.write_read(0x19, &[0x0F], &mut response);

        match result {
            Ok(()) => {
                assert!(response[0] == 0x33);
                defmt::info!("WHO_AM_I response as expected = {:x} ",response)
            },
            Err(e) => defmt::error!("I2C error: {:x}. response: {}", e, response),
        }

    }

    #[test]
    fn shtc3_measurement(ta: &mut TestArgs) {
        defmt::warn!("A SHTC3 sensor must be connected to the board on pins A9 (SCL) & A10 (SDA) for this test to work");

        let mut response: [u8; 6] = [0; 6];


        // send wake up command
        let result = ta.i2c.write(0x70, &[0x35, 0x17]);

        match result {
            Ok(()) => defmt::info!("SHTC3 was woken up"),
            Err(e) => defmt::error!("I2C error: {}", e),
        }

        // get sensor ID
        let result = ta.i2c.write_read(0x70, &[0xEF, 0xC8], &mut response);

        match result {
            Ok(()) => defmt::info!("SHTC3 ID = {:x} ",response),
            Err(e) => defmt::error!("I2C error: {}", e)
        }

        // get measurement read relative humditiy first and enable clock stretching
        let result = ta.i2c.write_read(0x70, &[0x5C, 0x24],&mut response);

        match result {
            Ok(()) => {
                defmt::info!("response:{:x}",response);
                let relative_humidity = 100.0*((response[1] as u16 + ((response[0] as u16) << 8)) as f32)/65535.0;
                let temperature = ((response[4] as u16 + ((response[3] as u16) << 8)) as f32)*175.0/65535.0 - 45.0;
                defmt::info!("rh {}%", relative_humidity);
                defmt::info!("temp {}°C", temperature);
            },
            Err(e) => defmt::error!("I2C error: {}", e),
        }

    }

}

