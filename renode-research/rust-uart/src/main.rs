//! STM32F4 Discovery UART Example in Rust
//!
//! This is a port of the C example from the Interrupt Memfault Renode tutorial.
//! It demonstrates:
//! - UART2 output at 115200 baud
//! - GPIO button input on PA0 (User Button)
//! - LED on PD12

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::{
    pac,
    prelude::*,
    serial::{Config, Serial},
};

/// Write a string to UART, converting \n to \r\n
fn uart_write_str<W: core::fmt::Write>(uart: &mut W, s: &str) {
    for c in s.chars() {
        if c == '\n' {
            let _ = uart.write_char('\r');
        }
        let _ = uart.write_char(c);
    }
}

#[entry]
fn main() -> ! {
    // Take ownership of the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Set up the system clocks
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // GPIO ports
    let gpioa = dp.GPIOA.split();
    let gpiod = dp.GPIOD.split();

    // Configure LED on PD12 as output
    let mut _led = gpiod.pd12.into_push_pull_output();

    // Configure User Button on PA0 as input (active high on STM32F4 Discovery)
    let button = gpioa.pa0.into_floating_input();

    // Configure USART2 pins
    // PA2 = TX (Alternate Function 7)
    let tx_pin = gpioa.pa2;

    // Set up USART2 at 115200 baud, TX only
    let mut serial = Serial::tx(
        dp.USART2,
        tx_pin,
        Config::default()
            .baudrate(115200.bps())
            .wordlength_8()
            .parity_none(),
        &clocks,
    )
    .unwrap();

    // Print hello world
    uart_write_str(&mut serial, "hello world!\n");

    // Button state tracking
    let mut button_is_pressed = false;

    // Main loop - detect button press/release
    loop {
        let button_state = button.is_high();

        if !button_is_pressed && button_state {
            // Button just pressed
            button_is_pressed = true;
        } else if button_is_pressed && !button_state {
            // Button just released
            uart_write_str(&mut serial, "button pressed\n");
            button_is_pressed = false;
        }
    }
}
