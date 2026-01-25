//! STM32F3 Discovery UART Example in Rust
//!
//! This is a port of the UART example for STM32F3 Discovery board.
//! It demonstrates:
//! - USART1 output at 115200 baud (PA9 TX, PA10 RX)
//! - GPIO button input on PA0 (User Button)
//! - LED on PE9

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f3xx_hal::{
    pac,
    prelude::*,
    serial::{Serial, config::Config},
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

    // Set up the system clocks using HSI (8 MHz internal oscillator)
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // GPIO ports
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    // Configure LED on PE9 as output
    let mut _led = gpioe.pe9.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    // Configure User Button on PA0 as input (active high on STM32F3 Discovery)
    let button = gpioa.pa0.into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);

    // Configure USART1 pins
    // PA9 = TX, PA10 = RX (Alternate Function 7)
    let tx_pin = gpioa.pa9.into_af_push_pull::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
    let rx_pin = gpioa.pa10.into_af_push_pull::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);

    // Set up USART1 at 115200 baud
    let mut serial = Serial::new(
        dp.USART1,
        (tx_pin, rx_pin),
        Config::default().baudrate(115200.Bd()),
        clocks,
        &mut rcc.apb2,
    );

    // Print hello world
    uart_write_str(&mut serial, "hello world!\n");

    // Turn on LED
    _led.set_high().ok();

    // Button state tracking
    let mut button_is_pressed = false;

    // Main loop - detect button press/release
    loop {
        let button_state = button.is_high().unwrap_or(false);

        if !button_is_pressed && button_state {
            // Button just pressed
            button_is_pressed = true;
        } else if button_is_pressed && !button_state {
            // Button just released
            uart_write_str(&mut serial, "button pressed\n");
            button_is_pressed = false;
            // Toggle LED
            _led.toggle().ok();
        }
    }
}
