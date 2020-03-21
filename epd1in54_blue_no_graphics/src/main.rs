#![no_main]
#![no_std]

// set the panic handler
#[allow(unused_imports)]
use panic_semihosting;

use cortex_m_rt::entry;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::{delay, spi};

use embedded_hal::prelude::*;
use epd_waveshare::{epd1in54::EPD1in54, prelude::*};

// activate spi, gpio in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues

#[entry]
fn main() -> ! {
    let core = cortex_m::Peripherals::take().unwrap();
    let device = stm32f1xx_hal::stm32::Peripherals::take().unwrap();
    let mut rcc = device.RCC.constrain();
    let mut flash = device.FLASH.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);

    let mut delay = delay::Delay::new(core.SYST, clocks);

    // spi setup
    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14;
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
    let mut spi = spi::Spi::spi2(
        device.SPI2,
        (sck, miso, mosi),
        epd_waveshare::SPI_MODE,
        4.mhz(),
        clocks,
        &mut rcc.apb1,
    );
    // epd setup
    let mut epd = epd_waveshare::epd1in54::EPD1in54::new(
        &mut spi,
        gpiob.pb12.into_push_pull_output(&mut gpiob.crh),
        gpioa.pa10.into_floating_input(&mut gpioa.crh),
        gpioa.pa8.into_push_pull_output(&mut gpioa.crh),
        gpioa.pa9.into_push_pull_output(&mut gpioa.crh),
        &mut delay,
    )
    .unwrap();

    // Clear the full screen
    epd.clear_frame(&mut spi).unwrap();
    epd.display_frame(&mut spi).unwrap();

    // Speeddemo
    epd.set_lut(&mut spi, Some(RefreshLUT::QUICK)).unwrap();
    let small_buffer = [Color::Black.get_byte_value(); 32]; //16x16
    let number_of_runs = 1;
    for i in 0..number_of_runs {
        let offset = i * 8 % 150;
        epd.update_partial_frame(&mut spi, &small_buffer, 25 + offset, 25 + offset, 16, 16)
            .unwrap();
        epd.display_frame(&mut spi).unwrap();
    }

    // Clear the full screen
    epd.clear_frame(&mut spi).unwrap();
    epd.display_frame(&mut spi).unwrap();

    // Draw some squares
    let small_buffer = [Color::Black.get_byte_value(); 3200]; //160x160
    epd.update_partial_frame(&mut spi, &small_buffer, 20, 20, 160, 160)
        .unwrap();

    let small_buffer = [Color::White.get_byte_value(); 800]; //80x80
    epd.update_partial_frame(&mut spi, &small_buffer, 60, 60, 80, 80)
        .unwrap();

    let small_buffer = [Color::Black.get_byte_value(); 8]; //8x8
    epd.update_partial_frame(&mut spi, &small_buffer, 96, 96, 8, 8)
        .unwrap();

    // Display updated frame
    epd.display_frame(&mut spi).unwrap();
    delay.delay_ms(5000u16);

    // Set the EPD to sleep
    epd.sleep(&mut spi).unwrap();

    loop {
        // sleep
        cortex_m::asm::wfi();
    }
}
