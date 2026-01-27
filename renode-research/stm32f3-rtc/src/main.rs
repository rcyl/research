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
use stm32f3_common::{constants, delay, uart_write_hex, uart_write_str};
use stm32f3xx_hal::{
    pac,
    prelude::*,
    serial::{config::Config as UartConfig, Serial},
};

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

    uart_write_str(&mut serial, "RTC Peripheral Test\n");

    // Get peripheral pointers via PAC
    let rtc = unsafe { &*pac::RTC::ptr() };
    let pwr = unsafe { &*pac::PWR::ptr() };
    let rcc_ptr = unsafe { &*pac::RCC::ptr() };

    // Initialize RTC
    // Enable PWR clock
    rcc_ptr.apb1enr.modify(|_, w| w.pwren().enabled());

    // Enable access to backup domain
    pwr.cr.modify(|_, w| w.dbp().set_bit());

    // Enable LSI and select as RTC clock source
    // Enable RTC clock, select LSI (bits 9:8 = 10)
    rcc_ptr.bdcr.modify(|_, w| {
        w.rtcen()
            .enabled()
            .rtcsel()
            .lsi()
    });

    delay(constants::MEDIUM_DELAY);

    // Disable RTC write protection
    rtc.wpr.write(|w| w.key().bits(0xCA));
    rtc.wpr.write(|w| w.key().bits(0x53));

    // Enter initialization mode
    rtc.isr.modify(|_, w| w.init().init_mode());

    // Wait for INITF flag
    let mut timeout = constants::INIT_TIMEOUT;
    while rtc.isr.read().initf().is_not_allowed() && timeout > 0 {
        timeout -= 1;
        delay(10);
    }

    uart_write_str(&mut serial, "RTC initialized\n");
    led.set_high().ok();

    // Set time to 12:30:00
    let hours: u8 = 12;
    let minutes: u8 = 30;
    let seconds: u8 = 0;

    // Set time register (BCD format)
    rtc.tr.write(|w| {
        w.ht()
            .bits(bin_to_bcd(hours) >> 4)
            .hu()
            .bits(bin_to_bcd(hours) & 0x0F)
            .mnt()
            .bits(bin_to_bcd(minutes) >> 4)
            .mnu()
            .bits(bin_to_bcd(minutes) & 0x0F)
            .st()
            .bits(bin_to_bcd(seconds) >> 4)
            .su()
            .bits(bin_to_bcd(seconds) & 0x0F)
    });

    // Exit initialization mode
    rtc.isr.modify(|_, w| w.init().free_running_mode());

    // Re-enable write protection
    rtc.wpr.write(|w| w.key().bits(0xFF));

    uart_write_str(&mut serial, "Time set: ");
    uart_write_hex(&mut serial, hours);
    uart_write_str(&mut serial, ":");
    uart_write_hex(&mut serial, minutes);
    uart_write_str(&mut serial, ":");
    uart_write_hex(&mut serial, seconds);
    uart_write_str(&mut serial, "\n");

    // Small delay to let time advance
    delay(constants::VERY_LONG_DELAY);

    // Read time back
    let tr_read = rtc.tr.read();

    let hours_read = bcd_to_bin((tr_read.ht().bits() << 4) | tr_read.hu().bits());
    let minutes_read = bcd_to_bin((tr_read.mnt().bits() << 4) | tr_read.mnu().bits());
    let seconds_read = bcd_to_bin((tr_read.st().bits() << 4) | tr_read.su().bits());

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
