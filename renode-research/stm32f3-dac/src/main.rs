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
use stm32f3_common::{constants, delay, uart_write_hex, uart_write_hex16, uart_write_str};
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

    uart_write_str(&mut serial, "DAC Peripheral Test\n");

    // Get peripheral pointers via PAC
    let dac1 = unsafe { &*pac::DAC1::ptr() };
    let rcc_ptr = unsafe { &*pac::RCC::ptr() };

    // Enable DAC clock
    rcc_ptr.apb1enr.modify(|_, w| w.dac1en().enabled());
    delay(constants::STABILIZATION_DELAY);

    uart_write_str(&mut serial, "DAC clock enabled\n");

    // Configure DAC outputs (PA4 = DAC1_OUT1, PA5 = DAC1_OUT2)
    // Set PA4 and PA5 to analog mode
    let _pa4 = gpioa
        .pa4
        .into_analog(&mut gpioa.moder, &mut gpioa.pupdr);
    let _pa5 = gpioa
        .pa5
        .into_analog(&mut gpioa.moder, &mut gpioa.pupdr);

    // Enable DAC channels
    dac1.cr.write(|w| w.en1().enabled().en2().enabled());
    delay(constants::STABILIZATION_DELAY);

    uart_write_str(&mut serial, "DAC channels enabled\n");

    // Test counters
    let mut tests_passed = 0u8;
    let mut tests_failed = 0u8;

    // ========================================
    // Test 1: DAC Channel 1 Output
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 1: DAC Channel 1 ---\n");

    // Write test value to channel 1 (12-bit: 0-4095)
    let test_value1: u16 = 2048; // Mid-scale
    uart_write_str(&mut serial, "Writing to CH1: 0x");
    uart_write_hex16(&mut serial, test_value1);
    uart_write_str(&mut serial, "\n");

    dac1.dhr12r1.write(|w| w.dacc1dhr().bits(test_value1));
    delay(constants::STABILIZATION_DELAY);

    // Read back from DOR1
    let dor1 = dac1.dor1.read().dacc1dor().bits();
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

    // ========================================
    // Test 2: DAC Channel 2 Output
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 2: DAC Channel 2 ---\n");

    // Write test value to channel 2
    let test_value2: u16 = 3072; // 75% scale
    uart_write_str(&mut serial, "Writing to CH2: 0x");
    uart_write_hex16(&mut serial, test_value2);
    uart_write_str(&mut serial, "\n");

    dac1.dhr12r2.write(|w| w.dacc2dhr().bits(test_value2));
    delay(constants::STABILIZATION_DELAY);

    // Read back from DOR2
    let dor2 = dac1.dor2.read().dacc2dor().bits();
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

    // ========================================
    // Test 3: DAC Value Range Test
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 3: DAC Value Range ---\n");

    let test_values: [u16; 3] = [0, 2047, 4095]; // Min, mid, max
    let mut range_pass = true;

    for val in test_values.iter() {
        dac1.dhr12r1.write(|w| w.dacc1dhr().bits(*val));
        delay(50);

        let readback = dac1.dor1.read().dacc1dor().bits();
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
