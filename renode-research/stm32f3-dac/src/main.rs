//! STM32F3 DAC (Digital-to-Analog Converter) Test
//!
//! This tests the DAC functionality:
//! - Enable DAC channels
//! - Write values to DAC channel 1 and 2
//! - Verify DOR output register values
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

/// Write a 16-bit hex value to UART
fn uart_write_hex16<W: core::fmt::Write>(uart: &mut W, value: u16) {
    uart_write_hex(uart, (value >> 8) as u8);
    uart_write_hex(uart, (value & 0xFF) as u8);
}

/// Simple delay loop
fn delay(cycles: u32) {
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}

/// DAC Register offsets (STM32F3)
const DAC_CR: u32 = 0x00;       // Control register
const DAC_DHR12R1: u32 = 0x08;  // Channel 1 12-bit right-aligned data
const DAC_DHR12R2: u32 = 0x14;  // Channel 2 12-bit right-aligned data
const DAC_DOR1: u32 = 0x2C;     // Channel 1 data output register
const DAC_DOR2: u32 = 0x30;     // Channel 2 data output register

/// DAC base address
const DAC_BASE: u32 = 0x40007400;

/// RCC base address
const RCC_BASE: u32 = 0x40021000;
const RCC_APB1ENR: u32 = 0x1C;

/// Write to DAC register
unsafe fn dac_write(offset: u32, value: u32) {
    let addr = (DAC_BASE + offset) as *mut u32;
    core::ptr::write_volatile(addr, value);
}

/// Read from DAC register
unsafe fn dac_read(offset: u32) -> u32 {
    let addr = (DAC_BASE + offset) as *const u32;
    core::ptr::read_volatile(addr)
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

    uart_write_str(&mut serial, "DAC Peripheral Test\n");

    // Enable DAC clock (bit 29 of APB1ENR)
    unsafe {
        let rcc_ptr = RCC_BASE as *mut u32;
        let apb1enr = core::ptr::read_volatile(rcc_ptr.offset((RCC_APB1ENR / 4) as isize));
        core::ptr::write_volatile(rcc_ptr.offset((RCC_APB1ENR / 4) as isize), apb1enr | (1 << 29));
    }
    delay(100);

    uart_write_str(&mut serial, "DAC clock enabled\n");

    // Configure DAC outputs (PA4 = DAC1, PA5 = DAC2)
    // Set PA4 and PA5 to analog mode
    let _pa4 = gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);
    let _pa5 = gpioa.pa5.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);

    // Enable DAC channels
    // CR: EN1 (bit 0) = enable channel 1, EN2 (bit 16) = enable channel 2
    unsafe {
        dac_write(DAC_CR, (1 << 0) | (1 << 16));
    }
    delay(100);

    uart_write_str(&mut serial, "DAC channels enabled\n");

    // Test counters
    let mut tests_passed = 0u8;
    let mut tests_failed = 0u8;

    // ========================================
    // Test 1: DAC Channel 1 Output
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 1: DAC Channel 1 ---\n");

    unsafe {
        // Write test value to channel 1 (12-bit: 0-4095)
        let test_value1: u16 = 2048; // Mid-scale
        uart_write_str(&mut serial, "Writing to CH1: 0x");
        uart_write_hex16(&mut serial, test_value1);
        uart_write_str(&mut serial, "\n");

        dac_write(DAC_DHR12R1, test_value1 as u32);
        delay(100);

        // Read back from DOR1
        let dor1 = dac_read(DAC_DOR1) as u16;
        uart_write_str(&mut serial, "DOR1 readback: 0x");
        uart_write_hex16(&mut serial, dor1);
        uart_write_str(&mut serial, "\n");

        // Verify the value was written
        if dor1 == test_value1 {
            uart_write_str(&mut serial, "DAC Channel 1: PASS\n");
            tests_passed += 1;
        } else {
            uart_write_str(&mut serial, "DAC Channel 1: FAIL\n");
            tests_failed += 1;
        }
    }

    // ========================================
    // Test 2: DAC Channel 2 Output
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 2: DAC Channel 2 ---\n");

    unsafe {
        // Write test value to channel 2
        let test_value2: u16 = 3072; // 75% scale
        uart_write_str(&mut serial, "Writing to CH2: 0x");
        uart_write_hex16(&mut serial, test_value2);
        uart_write_str(&mut serial, "\n");

        dac_write(DAC_DHR12R2, test_value2 as u32);
        delay(100);

        // Read back from DOR2
        let dor2 = dac_read(DAC_DOR2) as u16;
        uart_write_str(&mut serial, "DOR2 readback: 0x");
        uart_write_hex16(&mut serial, dor2);
        uart_write_str(&mut serial, "\n");

        // Verify the value was written
        if dor2 == test_value2 {
            uart_write_str(&mut serial, "DAC Channel 2: PASS\n");
            tests_passed += 1;
        } else {
            uart_write_str(&mut serial, "DAC Channel 2: FAIL\n");
            tests_failed += 1;
        }
    }

    // ========================================
    // Test 3: DAC Value Range Test
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 3: DAC Value Range ---\n");

    unsafe {
        let test_values: [u16; 3] = [0, 2047, 4095]; // Min, mid, max
        let mut range_pass = true;

        for val in test_values.iter() {
            dac_write(DAC_DHR12R1, *val as u32);
            delay(50);

            let readback = dac_read(DAC_DOR1) as u16;
            uart_write_str(&mut serial, "Value ");
            uart_write_hex16(&mut serial, *val);
            uart_write_str(&mut serial, " -> ");
            uart_write_hex16(&mut serial, readback);

            if readback == *val {
                uart_write_str(&mut serial, " OK\n");
            } else {
                uart_write_str(&mut serial, " FAIL\n");
                range_pass = false;
            }
        }

        if range_pass {
            uart_write_str(&mut serial, "DAC Value Range: PASS\n");
            tests_passed += 1;
        } else {
            uart_write_str(&mut serial, "DAC Value Range: FAIL\n");
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
        uart_write_str(&mut serial, "DAC TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "DAC TEST FAILED\n");
        led.set_low().ok();
    }

    // Halt
    loop {
        cortex_m::asm::wfi();
    }
}
