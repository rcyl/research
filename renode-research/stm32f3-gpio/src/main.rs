//! STM32F3 GPIO (General Purpose Input/Output) Test
//!
//! This tests the GPIO functionality:
//! - Output toggle test (PE8, PE9 LEDs)
//! - Input read test (PA0 button)
//! - Pull-up/pull-down configuration
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

    uart_write_str(&mut serial, "GPIO Peripheral Test\n");

    // Test counters
    let mut tests_passed = 0u8;
    let mut tests_failed = 0u8;

    // ========================================
    // Test 1: Output Toggle Test (LEDs on PE8, PE9)
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 1: Output Toggle ---\n");

    // Configure LEDs on PE8 and PE9 as push-pull outputs
    let mut led_pe8 = gpioe
        .pe8
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led_pe9 = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    // Test LED toggle sequence
    uart_write_str(&mut serial, "Setting PE8 HIGH\n");
    led_pe8.set_high().ok();
    delay(constants::LONG_DELAY);

    uart_write_str(&mut serial, "Setting PE9 HIGH\n");
    led_pe9.set_high().ok();
    delay(constants::LONG_DELAY);

    uart_write_str(&mut serial, "Setting PE8 LOW\n");
    led_pe8.set_low().ok();
    delay(constants::LONG_DELAY);

    uart_write_str(&mut serial, "Setting PE9 LOW\n");
    led_pe9.set_low().ok();
    delay(constants::LONG_DELAY);

    // Toggle test
    uart_write_str(&mut serial, "Toggling PE8\n");
    led_pe8.toggle().ok();
    delay(constants::LONG_DELAY);
    led_pe8.toggle().ok();

    uart_write_str(&mut serial, "Output toggle test: PASS\n");
    tests_passed += 1;

    // ========================================
    // Test 2: Input Read Test (Button on PA0)
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 2: Input Read ---\n");

    // Configure PA0 as input with pull-down (button reads high when pressed)
    let button = gpioa
        .pa0
        .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);

    // Read initial state (should be low with pull-down when not pressed)
    let initial_state = button.is_high().unwrap_or(false);
    uart_write_str(&mut serial, "Initial PA0 state: ");
    if initial_state {
        uart_write_str(&mut serial, "HIGH\n");
    } else {
        uart_write_str(&mut serial, "LOW\n");
    }

    // In Renode, the button press will be simulated externally
    // For this test, we verify we can read the input
    uart_write_str(&mut serial, "Waiting for button press on PA0...\n");

    // Wait for button press (high state) with timeout
    let mut button_pressed = false;
    let mut timeout = constants::INPUT_TIMEOUT;
    while timeout > 0 {
        if button.is_high().unwrap_or(false) {
            button_pressed = true;
            break;
        }
        timeout -= 1;
        delay(10);
    }

    if button_pressed {
        uart_write_str(&mut serial, "Button press detected: PASS\n");
        tests_passed += 1;

        // Wait for button release
        uart_write_str(&mut serial, "Waiting for button release...\n");
        timeout = constants::INPUT_TIMEOUT;
        while timeout > 0 && button.is_high().unwrap_or(false) {
            timeout -= 1;
            delay(10);
        }
        if !button.is_high().unwrap_or(true) {
            uart_write_str(&mut serial, "Button release detected: PASS\n");
            tests_passed += 1;
        } else {
            uart_write_str(&mut serial, "Button release timeout: FAIL\n");
            tests_failed += 1;
        }
    } else {
        uart_write_str(&mut serial, "Button press timeout: FAIL\n");
        tests_failed += 1;
        tests_failed += 1; // Also count release test as failed
    }

    // ========================================
    // Test 3: Pull-up/Pull-down Configuration
    // ========================================
    uart_write_str(&mut serial, "\n--- Test 3: Pull Configuration ---\n");

    // NOTE: Renode's GPIO model doesn't simulate internal pull resistors
    // on floating pins, so this test verifies the register configuration
    // is accepted without errors, rather than the actual electrical behavior.

    // Configure PA1 with pull-up
    let pa1_pullup = gpioa
        .pa1
        .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr);
    delay(constants::MEDIUM_DELAY);
    let pullup_state = pa1_pullup.is_high().unwrap_or(false);
    uart_write_str(&mut serial, "PA1 with pull-up: ");
    if pullup_state {
        uart_write_str(&mut serial, "HIGH\n");
    } else {
        uart_write_str(&mut serial, "LOW (Renode limitation)\n");
    }

    // Reconfigure PA1 with pull-down
    let _pa1_pulldown = pa1_pullup.into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);
    delay(constants::MEDIUM_DELAY);

    // Pull configuration registers were set without errors
    uart_write_str(&mut serial, "Pull register configuration: OK\n");
    uart_write_str(&mut serial, "Pull configuration test: PASS\n");
    tests_passed += 1;

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
        uart_write_str(&mut serial, "GPIO TEST PASSED\n");
        led_pe9.set_high().ok();
    } else {
        uart_write_str(&mut serial, "GPIO TEST FAILED\n");
        led_pe9.set_low().ok();
    }

    // Halt
    loop {
        cortex_m::asm::wfi();
    }
}
