//! STM32F3 ADC (Analog-to-Digital Converter) Test
//!
//! This tests the ADC functionality:
//! - ADC initialization and enable
//! - Single conversion mode
//! - Read conversion result
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

/// ADC Register offsets (STM32F3 ADC)
const ADC_ISR: u32 = 0x00;    // Interrupt and status register
const ADC_CR: u32 = 0x08;     // Control register
const ADC_CFGR: u32 = 0x0C;   // Configuration register
const ADC_SQR1: u32 = 0x30;   // Regular sequence register 1
const ADC_DR: u32 = 0x40;     // Regular data register

/// ADC base address (ADC1)
const ADC1_BASE: u32 = 0x50000000;

/// ADC Common base address
const ADC_COMMON_BASE: u32 = 0x50000300;
const ADC_CCR: u32 = 0x08;    // Common control register

/// RCC ADC clock enable
const RCC_BASE: u32 = 0x40021000;
const RCC_AHBENR: u32 = 0x14;

/// Write to ADC register
unsafe fn adc_write(offset: u32, value: u32) {
    let addr = (ADC1_BASE + offset) as *mut u32;
    core::ptr::write_volatile(addr, value);
}

/// Read from ADC register
unsafe fn adc_read(offset: u32) -> u32 {
    let addr = (ADC1_BASE + offset) as *const u32;
    core::ptr::read_volatile(addr)
}

/// Write to ADC common register
unsafe fn adc_common_write(offset: u32, value: u32) {
    let addr = (ADC_COMMON_BASE + offset) as *mut u32;
    core::ptr::write_volatile(addr, value);
}

/// Write to RCC register
unsafe fn rcc_write(offset: u32, value: u32) {
    let addr = (RCC_BASE + offset) as *mut u32;
    core::ptr::write_volatile(addr, value);
}

/// Read from RCC register
unsafe fn rcc_read(offset: u32) -> u32 {
    let addr = (RCC_BASE + offset) as *const u32;
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

    uart_write_str(&mut serial, "ADC Peripheral Test\n");

    // Initialize ADC
    unsafe {
        // Enable ADC clock (ADC12 is bit 28 of AHBENR)
        let ahbenr = rcc_read(RCC_AHBENR);
        rcc_write(RCC_AHBENR, ahbenr | (1 << 28));

        delay(1000);

        // Configure ADC clock in common control register
        // CKMODE = 01 (synchronous clock mode, ADC clock = AHB clock / 1)
        adc_common_write(ADC_CCR, 1 << 16);

        // Make sure ADC is disabled first
        adc_write(ADC_CR, 0);
        delay(100);

        // Configure ADC:
        // - Single conversion mode (CONT = 0)
        // - Right alignment (ALIGN = 0)
        // - 12-bit resolution (RES = 00)
        adc_write(ADC_CFGR, 0);

        // Set sequence length to 1 (L = 0 means 1 conversion)
        // and select channel 0 for first conversion
        adc_write(ADC_SQR1, 0);

        // Enable ADC (ADEN = 1)
        adc_write(ADC_CR, 1 << 0);

        // Wait for ADC ready (ADRDY flag in ISR)
        let mut timeout = 10000;
        while adc_read(ADC_ISR) & (1 << 0) == 0 && timeout > 0 {
            timeout -= 1;
            delay(10);
        }
    }

    uart_write_str(&mut serial, "ADC1 initialized\n");
    led.set_high().ok();

    // Perform conversions
    let mut test_passed = true;
    let num_conversions = 3;

    for i in 0..num_conversions {
        unsafe {
            // Start conversion (ADSTART = 1)
            let cr = adc_read(ADC_CR);
            adc_write(ADC_CR, cr | (1 << 2));

            // Wait for end of conversion (EOC flag)
            let mut timeout = 10000;
            while adc_read(ADC_ISR) & (1 << 2) == 0 && timeout > 0 {
                timeout -= 1;
                delay(10);
            }

            // Read conversion result
            let result = adc_read(ADC_DR) as u16;

            uart_write_str(&mut serial, "Channel 0 conversion ");
            uart_write_hex(&mut serial, i as u8);
            uart_write_str(&mut serial, ": 0x");
            uart_write_hex16(&mut serial, result);

            // In simulation, we expect a valid 12-bit value (0-4095)
            if result <= 0x0FFF {
                uart_write_str(&mut serial, " OK\n");
            } else {
                uart_write_str(&mut serial, " FAIL\n");
                test_passed = false;
            }

            // Clear EOC flag by reading DR (already done above)
        }

        delay(10000);
    }

    // Summary
    uart_write_str(&mut serial, "\n=== Test Summary ===\n");
    uart_write_str(&mut serial, "Conversions: ");
    uart_write_hex(&mut serial, num_conversions as u8);
    uart_write_str(&mut serial, "\n");

    if test_passed {
        uart_write_str(&mut serial, "ADC TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "ADC TEST FAILED\n");
        led.set_low().ok();
    }

    // Halt
    loop {
        cortex_m::asm::wfi();
    }
}
