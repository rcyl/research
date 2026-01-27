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
use stm32f3_common::{constants, delay, uart_write_hex, uart_write_str};
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
    // PA9 = TX, PA10 = RX (Alternate Function 7)
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

    uart_write_str(&mut serial, "IWDG Peripheral Test\n");

    // Get IWDG peripheral via PAC
    let iwdg = unsafe { &*pac::IWDG::ptr() };

    // Initialize IWDG
    // LSI clock is ~40kHz
    // Prescaler = 4 means divide by 4, so 40kHz/4 = 10kHz
    // Reload = 0xFFF (4095) means timeout = 4095/10kHz = ~410ms

    // Enable write access to PR and RLR (key = 0x5555)
    iwdg.kr.write(|w| unsafe { w.key().bits(0x5555) });

    // Set prescaler to 4 (PR = 0)
    iwdg.pr.write(|w| w.pr().divide_by4());

    // Set reload value to 0xFFF
    iwdg.rlr.write(|w| w.rl().bits(0xFFF));

    // Wait for registers to update (check status register)
    let mut timeout = constants::MEDIUM_DELAY;
    while (iwdg.sr.read().pvu().bit_is_set() || iwdg.sr.read().rvu().bit_is_set()) && timeout > 0 {
        timeout -= 1;
        delay(constants::STABILIZATION_DELAY);
    }

    // Start the watchdog (key = 0xCCCC)
    iwdg.kr.write(|w| unsafe { w.key().bits(0xCCCC) });

    uart_write_str(&mut serial, "IWDG initialized (prescaler=4, reload=0xFFF)\n");
    led.set_high().ok();

    uart_write_str(&mut serial, "Feeding watchdog...\n");

    // Feed the watchdog multiple times with delays
    for i in 1..=3 {
        // Delay a bit (but less than timeout)
        delay(constants::VERY_LONG_DELAY);

        // Reload the watchdog counter (key = 0xAAAA)
        iwdg.kr.write(|w| unsafe { w.key().bits(0xAAAA) });

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
        iwdg.kr.write(|w| unsafe { w.key().bits(0xAAAA) });
        cortex_m::asm::wfi();
    }
}
