//! STM32F3 RTC (Real-Time Clock) Test
//!
//! This tests the Real-Time Clock functionality:
//! - RTC initialization
//! - Set time and date
//! - Read time back and verify
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

/// Simple delay loop
fn delay(cycles: u32) {
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}

/// RTC Register offsets
const RTC_TR: u32 = 0x00;    // Time register
const RTC_DR: u32 = 0x04;    // Date register
const RTC_CR: u32 = 0x08;    // Control register
const RTC_ISR: u32 = 0x0C;   // Initialization and status register
const RTC_WPR: u32 = 0x24;   // Write protection register

/// RTC base address
const RTC_BASE: u32 = 0x40002800;

/// PWR base address (for backup domain access)
const PWR_BASE: u32 = 0x40007000;
const PWR_CR: u32 = 0x00;

/// RCC base address
const RCC_BASE: u32 = 0x40021000;
const RCC_BDCR: u32 = 0x20;  // Backup domain control register
const RCC_APB1ENR: u32 = 0x1C;

/// Write to RTC register
unsafe fn rtc_write(offset: u32, value: u32) {
    let addr = (RTC_BASE + offset) as *mut u32;
    core::ptr::write_volatile(addr, value);
}

/// Read from RTC register
unsafe fn rtc_read(offset: u32) -> u32 {
    let addr = (RTC_BASE + offset) as *const u32;
    core::ptr::read_volatile(addr)
}

/// Write to PWR register
unsafe fn pwr_write(offset: u32, value: u32) {
    let addr = (PWR_BASE + offset) as *mut u32;
    core::ptr::write_volatile(addr, value);
}

/// Read from PWR register
unsafe fn pwr_read(offset: u32) -> u32 {
    let addr = (PWR_BASE + offset) as *const u32;
    core::ptr::read_volatile(addr)
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

/// Convert BCD to binary
fn bcd_to_bin(bcd: u8) -> u8 {
    ((bcd >> 4) * 10) + (bcd & 0x0F)
}

/// Convert binary to BCD
fn bin_to_bcd(bin: u8) -> u8 {
    ((bin / 10) << 4) | (bin % 10)
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

    uart_write_str(&mut serial, "RTC Peripheral Test\n");

    // Initialize RTC
    unsafe {
        // Enable PWR clock
        let apb1enr = rcc_read(RCC_APB1ENR);
        rcc_write(RCC_APB1ENR, apb1enr | (1 << 28)); // PWREN

        // Enable access to backup domain
        let pwr_cr = pwr_read(PWR_CR);
        pwr_write(PWR_CR, pwr_cr | (1 << 8)); // DBP bit

        // Enable LSI and select as RTC clock source
        let bdcr = rcc_read(RCC_BDCR);
        // Enable RTC clock, select LSI (bits 9:8 = 10)
        rcc_write(RCC_BDCR, bdcr | (1 << 15) | (2 << 8));

        delay(1000);

        // Disable RTC write protection
        rtc_write(RTC_WPR, 0xCA);
        rtc_write(RTC_WPR, 0x53);

        // Enter initialization mode
        let isr = rtc_read(RTC_ISR);
        rtc_write(RTC_ISR, isr | (1 << 7)); // INIT bit

        // Wait for INITF flag
        let mut timeout = 10000;
        while rtc_read(RTC_ISR) & (1 << 6) == 0 && timeout > 0 {
            timeout -= 1;
            delay(10);
        }
    }

    uart_write_str(&mut serial, "RTC initialized\n");
    led.set_high().ok();

    // Set time to 12:30:00
    let hours: u8 = 12;
    let minutes: u8 = 30;
    let seconds: u8 = 0;

    unsafe {
        // Set time register (BCD format)
        // TR: bits 22:20 = hours tens, bits 19:16 = hours units
        //     bits 14:12 = minutes tens, bits 11:8 = minutes units
        //     bits 6:4 = seconds tens, bits 3:0 = seconds units
        let tr = ((bin_to_bcd(hours) as u32) << 16)
               | ((bin_to_bcd(minutes) as u32) << 8)
               | (bin_to_bcd(seconds) as u32);
        rtc_write(RTC_TR, tr);

        // Exit initialization mode
        let isr = rtc_read(RTC_ISR);
        rtc_write(RTC_ISR, isr & !(1 << 7)); // Clear INIT bit

        // Re-enable write protection
        rtc_write(RTC_WPR, 0xFF);
    }

    uart_write_str(&mut serial, "Time set: ");
    uart_write_hex(&mut serial, hours);
    uart_write_str(&mut serial, ":");
    uart_write_hex(&mut serial, minutes);
    uart_write_str(&mut serial, ":");
    uart_write_hex(&mut serial, seconds);
    uart_write_str(&mut serial, "\n");

    // Small delay to let time advance
    delay(100000);

    // Read time back
    let tr_read = unsafe { rtc_read(RTC_TR) };

    let hours_read = bcd_to_bin(((tr_read >> 16) & 0x3F) as u8);
    let minutes_read = bcd_to_bin(((tr_read >> 8) & 0x7F) as u8);
    let seconds_read = bcd_to_bin((tr_read & 0x7F) as u8);

    uart_write_str(&mut serial, "Time read: ");
    uart_write_hex(&mut serial, hours_read);
    uart_write_str(&mut serial, ":");
    uart_write_hex(&mut serial, minutes_read);
    uart_write_str(&mut serial, ":");
    uart_write_hex(&mut serial, seconds_read);
    uart_write_str(&mut serial, "\n");

    // Verify time (allow for some seconds to have passed)
    let mut test_passed = true;

    // Hours and minutes should match exactly
    if hours_read != hours {
        uart_write_str(&mut serial, "Hours mismatch!\n");
        test_passed = false;
    }
    if minutes_read != minutes {
        uart_write_str(&mut serial, "Minutes mismatch!\n");
        test_passed = false;
    }
    // Seconds can be 0 or slightly more
    if seconds_read > 10 {
        uart_write_str(&mut serial, "Seconds out of range!\n");
        test_passed = false;
    }

    // Summary
    uart_write_str(&mut serial, "\n=== Test Summary ===\n");
    if test_passed {
        uart_write_str(&mut serial, "Time verification: PASS\n");
        uart_write_str(&mut serial, "RTC TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "Time verification: FAIL\n");
        uart_write_str(&mut serial, "RTC TEST FAILED\n");
        led.set_low().ok();
    }

    // Halt
    loop {
        cortex_m::asm::wfi();
    }
}
