#q![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use stm32f3xx_hal as hal;

use hal::delay::Delay;
use hal::pac;
use hal::prelude::*;
use hal::spi::Spi;

use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

use mfrc522::{Mfrc522, Uid};

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);
    let mut gpiod = dp.GPIOD.split(&mut rcc.ahb);

    // clock configuration
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);
    // Configure pins for SPI
    let nss = gpiod //nss/sda =pd0
        .pd0
        .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);
    let sck = gpioc         //sck = pc10
        .pc10
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
    let miso = gpioc            //miso = pc11
        .pc11
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
    let mosi = gpioc            //mosi = pc12
        .pc12
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);

    // defaults to using MODE_0
    let spi = Spi::new(dp.SPI3, (sck, miso, mosi), 1.MHz(), clocks, &mut rcc.apb1);

    let mut timer = Delay::new(cp.SYST, clocks);

    let mut mfrc522 = Mfrc522::new(spi, nss).expect("could not create MFRC522");
    match mfrc522.version() {
        Ok(version) => defmt::info!("version {:x}", version),
        Err(_e) => defmt::error!("version error"),
    }

    let write = false;
    loop {
        match mfrc522.wupa() {
            Ok(atqa) => {
                defmt::info!("new card detected");
                match mfrc522.select(&atqa) {
                    Ok(ref uid @ Uid::Single(ref inner)) => {
                        defmt::info!("card uid {=[?]}", inner.as_bytes());
                        handle_card(&mut mfrc522, &uid, write);
                    }
                    Ok(ref uid @ Uid::Double(ref inner)) => {
                        defmt::info!("card double uid {=[?]}", inner.as_bytes());
                        handle_card(&mut mfrc522, &uid, write);
                    }
                    Ok(_) => defmt::info!("got other uid size"),
                    Err(e) => {
                        defmt::error!("Select error");
                        print_err(&e);
                    }
                }
            }
            Err(e) => {
                defmt::error!("WUPA error");
                print_err(&e);
            }
        }
        timer.delay_ms(1000u32);
    }
}

fn handle_card<E, SPI, NSS>(mfrc522: &mut Mfrc522<SPI, NSS>, uid: &Uid, write: bool)
where
    SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
    NSS: OutputPin,
{
    let key = [0xFF; 6];
    let buffer = [
        0xDE, 0x42, 0xAD, 0x42, 0xBE, 0x42, 0xEF, 0x42, 0xCA, 0x42, 0xFE, 0x42, 0xBA, 0x42, 0xBE,
        0x42,
    ];
    if mfrc522.mf_authenticate(uid, 1, &key).is_ok() {
        if write {
            match mfrc522.mf_write(1, buffer) {
                Ok(_) => {
                    defmt::info!("write success");
                }
                Err(e) => {
                    defmt::error!("error during read");
                    print_err(&e);
                }
            }
        } else {
            match mfrc522.mf_read(1) {
                Ok(data) => defmt::info!("read {=[?]}", data),
                Err(e) => {
                    defmt::error!("error during read");
                    print_err(&e);
                }
            }
        }
    } else {
        defmt::warn!("Could not authenticate");
    }

    if mfrc522.hlta().is_err() {
        defmt::error!("Could not halt");
    }
    if mfrc522.stop_crypto1().is_err() {
        defmt::error!("Could not disable crypto1");
    }
}

fn print_err<E>(err: &mfrc522::Error<E>) {
    match err {
        mfrc522::Error::Bcc => defmt::error!("error BCC"),
        mfrc522::Error::BufferOverflow => defmt::error!("error BufferOverflow"),
        mfrc522::Error::Collision => defmt::error!("error Collision"),
        mfrc522::Error::Crc => defmt::error!("error Crc"),
        mfrc522::Error::IncompleteFrame => defmt::error!("error Incomplete"),
        mfrc522::Error::NoRoom => defmt::error!("error NoRoom"),
        mfrc522::Error::Overheating => defmt::error!("error Overheating"),
        mfrc522::Error::Parity => defmt::error!("error Parity"),
        mfrc522::Error::Protocol => defmt::error!("error Protocol"),
        mfrc522::Error::Timeout => defmt::error!("error Timeout"),
        mfrc522::Error::Wr => defmt::error!("error Wr"),
        mfrc522::Error::Nak => defmt::error!("error Nak"),
        _ => defmt::error!("error SPI"),
    };
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
