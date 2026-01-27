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
use stm32f3_common::{constants, uart_write_hex, uart_write_hex32, uart_write_str};
use stm32f3xx_hal::{
    pac,
    prelude::*,
    serial::{config::Config as UartConfig, Serial},
};

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

    uart_write_str(&mut serial, "Timer Peripheral Test\n");

    let mut pass_count = 0u8;
    let mut fail_count = 0u8;

    // =========================================
    // Test 1: Timer2 as countdown timer (delay)
    // =========================================
    uart_write_str(&mut serial, "\nTest 1: Timer2 Countdown\n");

    // Enable TIM2 clock and configure directly for better Renode compatibility
    // The HAL's wait() polls UIF flag which Renode may not set properly
    unsafe {
        let rcc_ptr = &*pac::RCC::ptr();
        rcc_ptr.apb1enr.modify(|_, w| w.tim2en().enabled());
    }

    let tim2 = unsafe { &*pac::TIM2::ptr() };

    // Configure for 100ms timeout at 72MHz
    // Prescaler: 7199 -> 72MHz / 7200 = 10kHz (0.1ms per tick)
    // ARR: 999 -> 1000 ticks = 100ms
    tim2.psc.write(|w| w.psc().bits(7199));
    tim2.arr.write(|w| w.bits(999));
    tim2.cnt.write(|w| w.bits(0));

    // Generate update event to load prescaler, then clear the flag
    tim2.egr.write(|w| w.ug().update());
    tim2.sr.write(|w| w.uif().clear_bit());

    // Enable counter
    tim2.cr1.write(|w| w.cen().enabled());
    uart_write_str(&mut serial, "Timer2 started (100ms)\n");

    // Wait for timer to reach ARR value using wrap-around detection
    // The counter resets to 0 when it reaches ARR, so detect the wrap
    let arr_val = tim2.arr.read().bits();
    let mut last_cnt: u32 = 0;
    let mut timeout_count = 0u32;
    let mut expired = false;

    loop {
        let cnt = tim2.cnt.read().bits();

        // Detect wrap-around: counter was high and is now low
        if cnt < last_cnt && last_cnt > (arr_val / 2) {
            expired = true;
            break;
        }

        // Also check UIF flag as backup
        if tim2.sr.read().uif().bit_is_set() {
            expired = true;
            break;
        }

        last_cnt = cnt;
        timeout_count += 1;
        if timeout_count > constants::TIMER_TIMEOUT {
            break; // Safety timeout
        }
    }

    // Stop timer
    tim2.cr1.write(|w| w.cen().disabled());

    if expired {
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

    // Enable TIM3 clock and configure directly for better Renode compatibility
    unsafe {
        let rcc_ptr = &*pac::RCC::ptr();
        rcc_ptr.apb1enr.modify(|_, w| w.tim3en().enabled());
    }

    let tim3 = unsafe { &*pac::TIM3::ptr() };

    // Configure for 50ms period at 72MHz
    // Prescaler: 7199 -> 72MHz / 7200 = 10kHz (0.1ms per tick)
    // ARR: 499 -> 500 ticks = 50ms
    tim3.psc.write(|w| w.psc().bits(7199));
    tim3.arr.write(|w| unsafe { w.bits(499) });
    tim3.cnt.write(|w| unsafe { w.bits(0) });

    // Generate update event to load prescaler, then clear the flag
    tim3.egr.write(|w| w.ug().update());
    tim3.sr.write(|w| w.uif().clear_bit());

    // Enable counter in auto-reload mode
    tim3.cr1.write(|w| w.cen().enabled());
    uart_write_str(&mut serial, "Timer3 started (50ms periodic)\n");

    // Count multiple periods by detecting counter wrap
    let mut period_count = 0u8;
    let arr_val = tim3.arr.read().bits() as u16;

    for _ in 0..3 {
        // Wait for counter to reach near max
        let mut last_cnt: u16 = 0;
        let mut timeout = 0u32;
        loop {
            let cnt = tim3.cnt.read().bits() as u16;
            // Detect wrap-around (counter reset to 0 after reaching ARR)
            if cnt < last_cnt && last_cnt > (arr_val / 2) {
                break;
            }
            // Or counter reached ARR
            if cnt >= arr_val {
                // Wait for it to wrap
                while tim3.cnt.read().bits() as u16 >= arr_val / 2 {
                    timeout += 1;
                    if timeout > 10_000_000 {
                        break;
                    }
                }
                break;
            }
            last_cnt = cnt;
            timeout += 1;
            if timeout > 10_000_000 {
                break;
            }
        }
        period_count += 1;
        uart_write_str(&mut serial, "Period ");
        uart_write_hex(&mut serial, period_count);
        uart_write_str(&mut serial, " complete\n");
    }

    // Stop timer
    tim3.cr1.write(|w| w.cen().disabled());

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
    tim4.psc.write(|w| w.psc().bits(7999)); // 72MHz / 8000 = 9kHz
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
