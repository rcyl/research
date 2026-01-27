//! STM32F3 SPI Loopback Test in Rust
//!
//! This tests SPI1 functionality on the STM32F303:
//! - SPI1 configured with MOSI connected to MISO (loopback)
//! - Sends test bytes and verifies they are received correctly
//! - Reports results via USART1

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f3_common::{uart_write_hex, uart_write_str};
use stm32f3xx_hal::{
    pac,
    prelude::*,
    serial::{config::Config as UartConfig, Serial},
    spi::{config::Config as SpiConfig, Spi},
};

#[entry]
fn main() -> ! {
    // Take ownership of the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Set up the system clocks using HSI (8 MHz internal oscillator)
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // GPIO ports
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    // Configure LED on PE9 as output (for status indication)
    let mut led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    // Configure USART1 pins for debug output
    // PA9 = TX, PA10 = RX (Alternate Function 7)
    let tx_pin =
        gpioa
            .pa9
            .into_af_push_pull::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
    let rx_pin =
        gpioa
            .pa10
            .into_af_push_pull::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);

    // Set up USART1 at 115200 baud
    let mut serial = Serial::new(
        dp.USART1,
        (tx_pin, rx_pin),
        UartConfig::default().baudrate(115200.Bd()),
        clocks,
        &mut rcc.apb2,
    );

    uart_write_str(&mut serial, "SPI1 Loopback Test\n");

    // Configure SPI1 pins (Alternate Function 5)
    // PA5 = SCK, PA6 = MISO, PA7 = MOSI
    let sck =
        gpioa
            .pa5
            .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let miso =
        gpioa
            .pa6
            .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let mosi =
        gpioa
            .pa7
            .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

    // Configure SPI1 with default config (Mode 0, 1MHz)
    let spi_config = SpiConfig::default().frequency(1.MHz());

    let mut spi = Spi::new(
        dp.SPI1,
        (sck, miso, mosi),
        spi_config,
        clocks,
        &mut rcc.apb2,
    );

    uart_write_str(&mut serial, "SPI1 initialized\n");

    // Test data to send
    let test_data: [u8; 5] = [0xAA, 0x55, 0x12, 0x34, 0xFF];
    let mut pass_count = 0u8;
    let mut fail_count = 0u8;

    uart_write_str(&mut serial, "Starting loopback test...\n");

    // Perform loopback test for each byte
    for &tx_byte in test_data.iter() {
        // Transfer byte (send and receive simultaneously)
        let rx_byte = match spi.transfer(&mut [tx_byte]) {
            Ok(received) => received[0],
            Err(_) => 0x00,
        };

        // Report result
        uart_write_str(&mut serial, "TX: 0x");
        uart_write_hex(&mut serial, tx_byte);
        uart_write_str(&mut serial, " RX: 0x");
        uart_write_hex(&mut serial, rx_byte);

        if tx_byte == rx_byte {
            uart_write_str(&mut serial, " PASS\n");
            pass_count += 1;
            led.set_high().ok();
        } else {
            uart_write_str(&mut serial, " FAIL\n");
            fail_count += 1;
            led.set_low().ok();
        }
    }

    // Summary
    uart_write_str(&mut serial, "\n=== Test Summary ===\n");
    uart_write_str(&mut serial, "Passed: ");
    uart_write_hex(&mut serial, pass_count);
    uart_write_str(&mut serial, "\nFailed: ");
    uart_write_hex(&mut serial, fail_count);
    uart_write_str(&mut serial, "\n");

    if fail_count == 0 {
        uart_write_str(&mut serial, "SPI TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "SPI TEST FAILED\n");
        led.set_low().ok();
    }

    // Halt
    loop {
        cortex_m::asm::wfi();
    }
}
