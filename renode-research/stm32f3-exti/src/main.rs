//! STM32F3 EXTI (External Interrupt) Test
//!
//! This tests the EXTI functionality:
//! - Rising edge interrupt detection on PA0
//! - Falling edge interrupt detection
//! - Multiple interrupt count verification
//! - Reports results via USART1

#![no_std]
#![no_main]

use panic_halt as _;

use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use stm32f3xx_hal::{
    pac::{self, interrupt, EXTI, NVIC},
    prelude::*,
    serial::{Serial, config::Config as UartConfig},
};

// Global interrupt counter
static INTERRUPT_COUNT: AtomicU32 = AtomicU32::new(0);
static RISING_EDGE_COUNT: AtomicU32 = AtomicU32::new(0);
static FALLING_EDGE_COUNT: AtomicU32 = AtomicU32::new(0);

// Shared EXTI peripheral for clearing pending flags
static EXTI_PERIPHERAL: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));

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

/// EXTI0 interrupt handler (PA0)
#[interrupt]
fn EXTI0() {
    // Increment total interrupt count
    INTERRUPT_COUNT.fetch_add(1, Ordering::SeqCst);

    // Clear the pending flag
    cortex_m::interrupt::free(|cs| {
        if let Some(exti) = EXTI_PERIPHERAL.borrow(cs).borrow_mut().as_mut() {
            // Check which edge triggered (we configured both)
            // The pending register tells us an interrupt occurred
            if exti.pr1.read().pr0().bit_is_set() {
                // Clear the pending bit by writing 1
                exti.pr1.write(|w| w.pr0().set_bit());

                // We'll track this as a rising edge for simplicity
                // (In real hardware, you'd need additional logic to detect edge type)
                RISING_EDGE_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }
    });
}

#[entry]
fn main() -> ! {
    // Take ownership of the device peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

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

    uart_write_str(&mut serial, "EXTI Peripheral Test\n");

    // Configure PA0 as input for EXTI
    let _pa0 = gpioa.pa0.into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);

    // Enable SYSCFG clock for EXTI configuration
    // On STM32F3, SYSCFG is on APB2
    unsafe {
        let rcc_ptr = &*pac::RCC::ptr();
        rcc_ptr.apb2enr.modify(|_, w| w.syscfgen().enabled());
    }

    // Configure EXTI0 for PA0
    // By default, EXTI0 is already mapped to PA0 (SYSCFG_EXTICR1 = 0)
    // We just need to configure the edge detection and enable the interrupt

    let exti = dp.EXTI;

    // Configure rising edge trigger for line 0
    exti.rtsr1.modify(|_, w| w.tr0().enabled());

    // Configure falling edge trigger for line 0 (to detect both edges)
    exti.ftsr1.modify(|_, w| w.tr0().enabled());

    // Unmask interrupt for line 0
    exti.imr1.modify(|_, w| w.mr0().set_bit());

    // Store EXTI peripheral for use in interrupt handler
    cortex_m::interrupt::free(|cs| {
        EXTI_PERIPHERAL.borrow(cs).replace(Some(exti));
    });

    // Enable EXTI0 interrupt in NVIC
    unsafe {
        cp.NVIC.set_priority(pac::Interrupt::EXTI0, 1);
        NVIC::unmask(pac::Interrupt::EXTI0);
    }

    uart_write_str(&mut serial, "EXTI0 configured for PA0 (rising + falling edge)\n");
    led.set_high().ok();

    // Test counters
    let mut tests_passed = 0u8;
    let mut tests_failed = 0u8;

    // ========================================
    // Test 1: Rising Edge Interrupt
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 1: Rising Edge Interrupt ---\n");
    uart_write_str(&mut serial, "Waiting for button press (rising edge)...\n");

    let initial_count = INTERRUPT_COUNT.load(Ordering::SeqCst);

    // Wait for interrupt with timeout
    let mut timeout = 500000u32;
    while timeout > 0 && INTERRUPT_COUNT.load(Ordering::SeqCst) == initial_count {
        timeout -= 1;
        delay(10);
    }

    if INTERRUPT_COUNT.load(Ordering::SeqCst) > initial_count {
        uart_write_str(&mut serial, "Rising edge interrupt detected: PASS\n");
        tests_passed += 1;
        led.toggle().ok();
    } else {
        uart_write_str(&mut serial, "Rising edge interrupt timeout: FAIL\n");
        tests_failed += 1;
    }

    // ========================================
    // Test 2: Falling Edge Interrupt
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 2: Falling Edge Interrupt ---\n");
    uart_write_str(&mut serial, "Waiting for button release (falling edge)...\n");

    let count_before_release = INTERRUPT_COUNT.load(Ordering::SeqCst);

    // Wait for another interrupt (falling edge)
    timeout = 500000;
    while timeout > 0 && INTERRUPT_COUNT.load(Ordering::SeqCst) == count_before_release {
        timeout -= 1;
        delay(10);
    }

    if INTERRUPT_COUNT.load(Ordering::SeqCst) > count_before_release {
        uart_write_str(&mut serial, "Falling edge interrupt detected: PASS\n");
        tests_passed += 1;
        led.toggle().ok();
    } else {
        uart_write_str(&mut serial, "Falling edge interrupt timeout: FAIL\n");
        tests_failed += 1;
    }

    // ========================================
    // Test 3: Multiple Interrupt Count
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 3: Multiple Interrupt Count ---\n");
    uart_write_str(&mut serial, "Press button 2 more times...\n");

    let count_before_multi = INTERRUPT_COUNT.load(Ordering::SeqCst);
    let target_count = count_before_multi + 4; // 2 presses = 4 edges (2 rising + 2 falling)

    // Wait for 4 more interrupts
    timeout = 1000000;
    while timeout > 0 && INTERRUPT_COUNT.load(Ordering::SeqCst) < target_count {
        timeout -= 1;
        delay(10);
    }

    let final_count = INTERRUPT_COUNT.load(Ordering::SeqCst);
    uart_write_str(&mut serial, "Total interrupts: ");
    uart_write_hex(&mut serial, final_count as u8);
    uart_write_str(&mut serial, "\n");

    if final_count >= target_count {
        uart_write_str(&mut serial, "Multiple interrupt count: PASS\n");
        tests_passed += 1;
    } else {
        uart_write_str(&mut serial, "Multiple interrupt count: FAIL\n");
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
    uart_write_str(&mut serial, "Total interrupts: ");
    uart_write_hex(&mut serial, INTERRUPT_COUNT.load(Ordering::SeqCst) as u8);
    uart_write_str(&mut serial, "\n");

    if tests_failed == 0 {
        uart_write_str(&mut serial, "EXTI TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "EXTI TEST FAILED\n");
        led.set_low().ok();
    }

    // Halt
    loop {
        cortex_m::asm::wfi();
    }
}
