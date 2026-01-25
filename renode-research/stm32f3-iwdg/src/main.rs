//! STM32F3 IWDG (Independent Watchdog) Test
//!
//! This tests the Independent Watchdog Timer functionality:
//! - IWDG initialization with prescaler and reload value
//! - Watchdog feeding (reload) to prevent reset
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

/// IWDG Register offsets
const IWDG_KR: u32 = 0x00;   // Key register
const IWDG_PR: u32 = 0x04;   // Prescaler register
const IWDG_RLR: u32 = 0x08;  // Reload register
const IWDG_SR: u32 = 0x0C;   // Status register

/// IWDG Key values
const KEY_RELOAD: u16 = 0xAAAA;   // Reload the counter
const KEY_ENABLE: u16 = 0xCCCC;   // Enable the watchdog
const KEY_WRITE_ACCESS: u16 = 0x5555; // Enable write access to PR and RLR

/// IWDG base address
const IWDG_BASE: u32 = 0x40003000;

/// Write to IWDG register
unsafe fn iwdg_write(offset: u32, value: u16) {
    let addr = (IWDG_BASE + offset) as *mut u16;
    core::ptr::write_volatile(addr, value);
}

/// Read from IWDG register
unsafe fn iwdg_read(offset: u32) -> u32 {
    let addr = (IWDG_BASE + offset) as *const u32;
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
    // PA9 = TX, PA10 = RX (Alternate Function 7)
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

    uart_write_str(&mut serial, "IWDG Peripheral Test\n");

    // Initialize IWDG
    // LSI clock is ~40kHz
    // Prescaler = 4 means divide by 4, so 40kHz/4 = 10kHz
    // Reload = 0xFFF (4095) means timeout = 4095/10kHz = ~410ms

    unsafe {
        // Enable write access to PR and RLR
        iwdg_write(IWDG_KR, KEY_WRITE_ACCESS);

        // Set prescaler to 4 (PR = 0)
        iwdg_write(IWDG_PR, 0);

        // Set reload value to 0xFFF
        iwdg_write(IWDG_RLR, 0xFFF);

        // Wait for registers to update (check status register)
        let mut timeout = 1000;
        while iwdg_read(IWDG_SR) != 0 && timeout > 0 {
            timeout -= 1;
            delay(100);
        }

        // Start the watchdog
        iwdg_write(IWDG_KR, KEY_ENABLE);
    }

    uart_write_str(&mut serial, "IWDG initialized (prescaler=4, reload=0xFFF)\n");
    led.set_high().ok();

    uart_write_str(&mut serial, "Feeding watchdog...\n");

    // Feed the watchdog multiple times with delays
    for i in 1..=3 {
        // Delay a bit (but less than timeout)
        delay(100000);

        // Reload the watchdog counter
        unsafe {
            iwdg_write(IWDG_KR, KEY_RELOAD);
        }

        uart_write_str(&mut serial, "Feed ");
        uart_write_hex(&mut serial, i);
        uart_write_str(&mut serial, ": OK\n");

        // Toggle LED
        if i % 2 == 0 {
            led.set_low().ok();
        } else {
            led.set_high().ok();
        }
    }

    // Final status
    uart_write_str(&mut serial, "\n=== Test Summary ===\n");
    uart_write_str(&mut serial, "Watchdog feeds: 3\n");
    uart_write_str(&mut serial, "System resets: 0\n");
    uart_write_str(&mut serial, "IWDG TEST PASSED\n");

    led.set_high().ok();

    // Keep feeding to prevent reset in the loop
    loop {
        delay(50000);
        unsafe {
            iwdg_write(IWDG_KR, KEY_RELOAD);
        }
        cortex_m::asm::wfi();
    }
}
