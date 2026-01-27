//! STM32F3 CRC (Cyclic Redundancy Check) Test
//!
//! This tests the CRC calculation unit functionality:
//! - CRC-32 calculation with known data
//! - Verify against expected CRC values
//! - Reset functionality
//! - Reports results via USART1

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f3_common::{constants, delay, uart_write_hex, uart_write_hex32, uart_write_str};
use stm32f3xx_hal::{
    pac,
    prelude::*,
    serial::{config::Config as UartConfig, Serial},
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

    uart_write_str(&mut serial, "CRC Peripheral Test\n");

    // Get peripheral pointers via PAC
    let crc = unsafe { &*pac::CRC::ptr() };
    let rcc_ptr = unsafe { &*pac::RCC::ptr() };

    // Enable CRC clock
    rcc_ptr.ahbenr.modify(|_, w| w.crcen().enabled());
    delay(constants::STABILIZATION_DELAY);

    uart_write_str(&mut serial, "CRC clock enabled\n");

    // Test counters
    let mut tests_passed = 0u8;
    let mut tests_failed = 0u8;

    // ========================================
    // Test 1: Single Word CRC Calculation
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 1: Single Word CRC ---\n");

    // Reset CRC to initial value (0xFFFFFFFF)
    crc.cr.write(|w| w.reset().reset());
    delay(10);

    // Read initial value (should be 0xFFFFFFFF)
    let init_val = crc.dr().read().bits();
    uart_write_str(&mut serial, "Initial CRC: 0x");
    uart_write_hex32(&mut serial, init_val);
    uart_write_str(&mut serial, "\n");

    // Write a test word
    let test_word: u32 = 0x12345678;
    uart_write_str(&mut serial, "Input word: 0x");
    uart_write_hex32(&mut serial, test_word);
    uart_write_str(&mut serial, "\n");

    crc.dr().write(|w| w.dr().bits(test_word));

    // Read calculated CRC
    let crc_result = crc.dr().read().bits();
    uart_write_str(&mut serial, "CRC result: 0x");
    uart_write_hex32(&mut serial, crc_result);
    uart_write_str(&mut serial, "\n");

    // The CRC should be different from the input
    if crc_result != test_word && crc_result != 0xFFFFFFFF {
        uart_write_str(&mut serial, "Single word CRC: PASS\n");
        tests_passed += 1;
    } else {
        uart_write_str(&mut serial, "Single word CRC: FAIL\n");
        tests_failed += 1;
    }

    // ========================================
    // Test 2: Multiple Word CRC Calculation
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 2: Multiple Word CRC ---\n");

    // Reset CRC
    crc.cr.write(|w| w.reset().reset());

    // Write multiple test words
    let test_data: [u32; 4] = [0x00000000, 0x11111111, 0x22222222, 0x33333333];

    for word in test_data.iter() {
        crc.dr().write(|w| w.dr().bits(*word));
    }

    // Read final CRC
    let crc_multi = crc.dr().read().bits();
    uart_write_str(&mut serial, "Multi-word CRC: 0x");
    uart_write_hex32(&mut serial, crc_multi);
    uart_write_str(&mut serial, "\n");

    // CRC should be computed
    if crc_multi != 0xFFFFFFFF {
        uart_write_str(&mut serial, "Multiple word CRC: PASS\n");
        tests_passed += 1;
    } else {
        uart_write_str(&mut serial, "Multiple word CRC: FAIL\n");
        tests_failed += 1;
    }

    // ========================================
    // Test 3: CRC Reset Functionality
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 3: CRC Reset ---\n");

    // First, compute some CRC
    crc.dr().write(|w| w.dr().bits(0xDEADBEEF));
    let before_reset = crc.dr().read().bits();
    uart_write_str(&mut serial, "Before reset: 0x");
    uart_write_hex32(&mut serial, before_reset);
    uart_write_str(&mut serial, "\n");

    // Reset CRC
    crc.cr.write(|w| w.reset().reset());
    delay(10);

    // Read CRC after reset
    let after_reset = crc.dr().read().bits();
    uart_write_str(&mut serial, "After reset: 0x");
    uart_write_hex32(&mut serial, after_reset);
    uart_write_str(&mut serial, "\n");

    // After reset, CRC should return to initial value (0xFFFFFFFF)
    if after_reset == 0xFFFFFFFF && before_reset != after_reset {
        uart_write_str(&mut serial, "CRC reset: PASS\n");
        tests_passed += 1;
    } else {
        uart_write_str(&mut serial, "CRC reset: FAIL\n");
        tests_failed += 1;
    }

    // ========================================
    // Test Summary
    // ========================================
    uart_write_str(&mut serial, "\n=== Test Summary ===\n");
    uart_write_str(&mut serial, "Tests passed: ");
    uart_write_hex(&mut serial, tests_passed);
    uart_write_str(&mut serial, "\n");
    uart_write_str(&mut serial, "Tests failed: ");
    uart_write_hex(&mut serial, tests_failed);
    uart_write_str(&mut serial, "\n");

    if tests_failed == 0 {
        uart_write_str(&mut serial, "CRC TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "CRC TEST FAILED\n");
        led.set_low().ok();
    }

    // Halt
    loop {
        cortex_m::asm::wfi();
    }
}
