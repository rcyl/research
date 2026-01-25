//! STM32F3 Timer Peripheral Test in Rust
//!
//! This tests Timer functionality on the STM32F303:
//! - Timer2 configured as a basic counter
//! - Timer3 configured with periodic updates
//! - Verifies counter increments and timing
//! - Reports results via USART1

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f3xx_hal::{
    pac,
    prelude::*,
    serial::{Serial, config::Config as UartConfig},
    timer::Timer,
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
fn uart_write_hex32<W: core::fmt::Write>(uart: &mut W, val: u32) {
    uart_write_hex(uart, ((val >> 24) & 0xFF) as u8);
    uart_write_hex(uart, ((val >> 16) & 0xFF) as u8);
    uart_write_hex(uart, ((val >> 8) & 0xFF) as u8);
    uart_write_hex(uart, (val & 0xFF) as u8);
}

#[entry]
fn main() -> ! {
    // Take ownership of the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Set up the system clocks
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // GPIO ports
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    // Configure LED on PE9 as output
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

    uart_write_str(&mut serial, "Timer Peripheral Test\n");

    let mut pass_count = 0u8;
    let mut fail_count = 0u8;

    // =========================================
    // Test 1: Timer2 as countdown timer (delay)
    // =========================================
    uart_write_str(&mut serial, "\nTest 1: Timer2 Countdown\n");

    // Create a countdown timer with 1 second period
    let mut timer2 = Timer::new(dp.TIM2, clocks, &mut rcc.apb1);

    // Start a 100ms countdown
    timer2.start(100.milliseconds());
    uart_write_str(&mut serial, "Timer2 started (100ms)\n");

    // Wait for timer to expire
    let mut timeout_count = 0u32;
    while timer2.wait().is_err() {
        timeout_count += 1;
        if timeout_count > 1_000_000 {
            break; // Safety timeout
        }
    }

    if timeout_count < 1_000_000 {
        uart_write_str(&mut serial, "Timer2 expired: PASS\n");
        pass_count += 1;
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "Timer2 timeout: FAIL\n");
        fail_count += 1;
    }

    // =========================================
    // Test 2: Timer3 periodic mode
    // =========================================
    uart_write_str(&mut serial, "\nTest 2: Timer3 Periodic\n");

    let mut timer3 = Timer::new(dp.TIM3, clocks, &mut rcc.apb1);
    timer3.start(50.milliseconds());
    uart_write_str(&mut serial, "Timer3 started (50ms periodic)\n");

    // Count multiple periods
    let mut period_count = 0u8;
    for _ in 0..3 {
        while timer3.wait().is_err() {
            // Busy wait
        }
        period_count += 1;
        uart_write_str(&mut serial, "Period ");
        uart_write_hex(&mut serial, period_count);
        uart_write_str(&mut serial, " complete\n");
    }

    if period_count == 3 {
        uart_write_str(&mut serial, "Timer3 periodic: PASS\n");
        pass_count += 1;
    } else {
        uart_write_str(&mut serial, "Timer3 periodic: FAIL\n");
        fail_count += 1;
    }

    // =========================================
    // Test 3: Timer4 direct register access
    // =========================================
    uart_write_str(&mut serial, "\nTest 3: Timer4 Counter\n");

    // Enable TIM4 clock manually
    unsafe {
        let rcc_ptr = &*pac::RCC::ptr();
        rcc_ptr.apb1enr.modify(|_, w| w.tim4en().enabled());
    }

    // Configure TIM4 directly
    let tim4 = unsafe { &*pac::TIM4::ptr() };

    // Set prescaler and auto-reload
    tim4.psc.write(|w| unsafe { w.psc().bits(7999) }); // 72MHz / 8000 = 9kHz
    tim4.arr.write(|w| unsafe { w.bits(0xFFFF) }); // Max count
    tim4.cnt.write(|w| unsafe { w.bits(0) }); // Clear counter

    // Enable counter
    tim4.cr1.write(|w| w.cen().enabled());

    // Read counter a few times
    let cnt1 = tim4.cnt.read().bits();
    for _ in 0..10000 {
        cortex_m::asm::nop();
    }
    let cnt2 = tim4.cnt.read().bits();

    uart_write_str(&mut serial, "CNT1: 0x");
    uart_write_hex32(&mut serial, cnt1);
    uart_write_str(&mut serial, "\nCNT2: 0x");
    uart_write_hex32(&mut serial, cnt2);
    uart_write_str(&mut serial, "\n");

    if cnt2 > cnt1 {
        uart_write_str(&mut serial, "Counter incrementing: PASS\n");
        pass_count += 1;
    } else {
        uart_write_str(&mut serial, "Counter not incrementing: FAIL\n");
        fail_count += 1;
    }

    // Stop timer
    tim4.cr1.write(|w| w.cen().disabled());

    // =========================================
    // Summary
    // =========================================
    uart_write_str(&mut serial, "\n=== Test Summary ===\n");
    uart_write_str(&mut serial, "Passed: ");
    uart_write_hex(&mut serial, pass_count);
    uart_write_str(&mut serial, "\nFailed: ");
    uart_write_hex(&mut serial, fail_count);
    uart_write_str(&mut serial, "\n");

    if fail_count == 0 {
        uart_write_str(&mut serial, "TIMER TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "TIMER TEST FAILED\n");
        led.set_low().ok();
    }

    loop {
        cortex_m::asm::wfi();
    }
}
