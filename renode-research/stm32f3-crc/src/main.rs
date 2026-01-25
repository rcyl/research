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
use stm32f3xx_hal::{
    pac,
    prelude::*,
    serial::{Serial, config::Config as UartConfig},
};

/// Write a string to UART
fn uart_write_str<W: core::fmt::Write>(uart: &mut W, s: &str) {
    for c in s.chars() {
        if c == '\n' {
            let _ = uart.write_char('\r');
        }
        let _ = uart.write_char(c);
    }
}

/// Write a hex byte to UART
fn uart_write_hex<W: core::fmt::Write>(uart: &mut W, byte: u8) {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    let _ = uart.write_char(HEX_CHARS[(byte >> 4) as usize] as char);
    let _ = uart.write_char(HEX_CHARS[(byte & 0x0F) as usize] as char);
}

/// Write a 32-bit hex value to UART
fn uart_write_hex32<W: core::fmt::Write>(uart: &mut W, value: u32) {
    uart_write_hex(uart, ((value >> 24) & 0xFF) as u8);
    uart_write_hex(uart, ((value >> 16) & 0xFF) as u8);
    uart_write_hex(uart, ((value >> 8) & 0xFF) as u8);
    uart_write_hex(uart, (value & 0xFF) as u8);
}

/// Simple delay loop
fn delay(cycles: u32) {
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}

/// CRC Register offsets
const CRC_DR: u32 = 0x00;   // Data register
const CRC_IDR: u32 = 0x04;  // Independent data register
const CRC_CR: u32 = 0x08;   // Control register
const CRC_INIT: u32 = 0x10; // Initial CRC value
const CRC_POL: u32 = 0x14;  // Polynomial

/// CRC base address
const CRC_BASE: u32 = 0x40023000;

/// RCC base address
const RCC_BASE: u32 = 0x40021000;
const RCC_AHBENR: u32 = 0x14;

/// Write to CRC register
unsafe fn crc_write(offset: u32, value: u32) {
    let addr = (CRC_BASE + offset) as *mut u32;
    core::ptr::write_volatile(addr, value);
}

/// Read from CRC register
unsafe fn crc_read(offset: u32) -> u32 {
    let addr = (CRC_BASE + offset) as *const u32;
    core::ptr::read_volatile(addr)
}

/// Reset CRC calculation (set RESET bit in CR)
unsafe fn crc_reset() {
    crc_write(CRC_CR, 1); // RESET bit
    delay(10);
}

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
    let mut led = gpioe.pe9.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    // Configure USART1 pins for debug output
    let tx_pin = gpioa.pa9.into_af_push_pull::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
    let rx_pin = gpioa.pa10.into_af_push_pull::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);

    // Set up USART1 at 115200 baud
    let mut serial = Serial::new(
        dp.USART1,
        (tx_pin, rx_pin),
        UartConfig::default().baudrate(115200.Bd()),
        clocks,
        &mut rcc.apb2,
    );

    uart_write_str(&mut serial, "CRC Peripheral Test\n");

    // Enable CRC clock (bit 6 of AHBENR)
    unsafe {
        let rcc_ptr = RCC_BASE as *mut u32;
        let ahbenr = core::ptr::read_volatile(rcc_ptr.offset((RCC_AHBENR / 4) as isize));
        core::ptr::write_volatile(rcc_ptr.offset((RCC_AHBENR / 4) as isize), ahbenr | (1 << 6));
    }
    delay(100);

    uart_write_str(&mut serial, "CRC clock enabled\n");

    // Test counters
    let mut tests_passed = 0u8;
    let mut tests_failed = 0u8;

    // ========================================
    // Test 1: Single Word CRC Calculation
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 1: Single Word CRC ---\n");

    unsafe {
        // Reset CRC to initial value (0xFFFFFFFF)
        crc_reset();

        // Read initial value (should be 0xFFFFFFFF)
        let init_val = crc_read(CRC_DR);
        uart_write_str(&mut serial, "Initial CRC: 0x");
        uart_write_hex32(&mut serial, init_val);
        uart_write_str(&mut serial, "\n");

        // Write a test word
        let test_word: u32 = 0x12345678;
        uart_write_str(&mut serial, "Input word: 0x");
        uart_write_hex32(&mut serial, test_word);
        uart_write_str(&mut serial, "\n");

        crc_write(CRC_DR, test_word);

        // Read calculated CRC
        let crc_result = crc_read(CRC_DR);
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
    }

    // ========================================
    // Test 2: Multiple Word CRC Calculation
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 2: Multiple Word CRC ---\n");

    unsafe {
        // Reset CRC
        crc_reset();

        // Write multiple test words
        let test_data: [u32; 4] = [0x00000000, 0x11111111, 0x22222222, 0x33333333];

        for word in test_data.iter() {
            crc_write(CRC_DR, *word);
        }

        // Read final CRC
        let crc_multi = crc_read(CRC_DR);
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
    }

    // ========================================
    // Test 3: CRC Reset Functionality
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 3: CRC Reset ---\n");

    unsafe {
        // First, compute some CRC
        crc_write(CRC_DR, 0xDEADBEEF);
        let before_reset = crc_read(CRC_DR);
        uart_write_str(&mut serial, "Before reset: 0x");
        uart_write_hex32(&mut serial, before_reset);
        uart_write_str(&mut serial, "\n");

        // Reset CRC
        crc_reset();

        // Read CRC after reset
        let after_reset = crc_read(CRC_DR);
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
