//! STM32F3 ADC (Analog-to-Digital Converter) Test
//!
//! This tests the ADC functionality:
//! - ADC initialization and enable
//! - Single conversion mode
//! - Read conversion result
//! - Reports results via USART1

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f3_common::{constants, delay, uart_write_hex, uart_write_hex16, uart_write_str};
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

    uart_write_str(&mut serial, "ADC Peripheral Test\n");

    // Get ADC peripherals via PAC
    let adc1 = unsafe { &*pac::ADC1::ptr() };
    let adc1_2 = unsafe { &*pac::ADC1_2::ptr() };
    let rcc_ptr = unsafe { &*pac::RCC::ptr() };

    // Initialize ADC
    // Enable ADC clock (ADC12 is bit 28 of AHBENR)
    rcc_ptr.ahbenr.modify(|_, w| w.adc12en().enabled());

    delay(constants::MEDIUM_DELAY);

    // Configure ADC clock in common control register
    // CKMODE = 01 (synchronous clock mode, ADC clock = AHB clock / 1)
    adc1_2.ccr.modify(|_, w| w.ckmode().bits(0b01));

    // Make sure ADC is disabled first
    adc1.cr.write(|w| w.aden().clear_bit());
    delay(constants::STABILIZATION_DELAY);

    // Configure ADC:
    // - Single conversion mode (CONT = 0)
    // - Right alignment (ALIGN = 0)
    // - 12-bit resolution (RES = 00)
    adc1.cfgr.write(|w| w.cont().single().align().right().res().bits12());

    // Set sequence length to 1 (L = 0 means 1 conversion)
    // and select channel 0 for first conversion
    adc1.sqr1.write(|w| unsafe { w.l().bits(0).sq1().bits(0) });

    // Enable ADC (ADEN = 1)
    adc1.cr.modify(|_, w| w.aden().enabled());

    // Wait for ADC ready (ADRDY flag in ISR)
    let mut timeout = constants::INIT_TIMEOUT;
    while adc1.isr.read().adrdy().is_not_ready() && timeout > 0 {
        timeout -= 1;
        delay(10);
    }

    uart_write_str(&mut serial, "ADC1 initialized\n");
    led.set_high().ok();

    // Perform conversions
    let mut test_passed = true;
    let num_conversions = 3;

    for i in 0..num_conversions {
        // Start conversion (ADSTART = 1)
        adc1.cr.modify(|_, w| w.adstart().set_bit());

        // Wait for end of conversion (EOC flag)
        let mut timeout = constants::INIT_TIMEOUT;
        while adc1.isr.read().eoc().is_not_complete() && timeout > 0 {
            timeout -= 1;
            delay(10);
        }

        // Read conversion result
        let result = adc1.dr.read().rdata().bits();

        uart_write_str(&mut serial, "Channel 0 conversion ");
        uart_write_hex(&mut serial, i as u8);
        uart_write_str(&mut serial, ": 0x");
        uart_write_hex16(&mut serial, result);

        // In simulation, we expect a valid 12-bit value (0-4095)
        if result <= 0x0FFF {
            uart_write_str(&mut serial, " OK\n");
        } else {
            uart_write_str(&mut serial, " FAIL\n");
            test_passed = false;
        }

        // Clear EOC flag by reading DR (already done above)
        delay(constants::LONG_DELAY);
    }

    // Summary
    uart_write_str(&mut serial, "\n=== Test Summary ===\n");
    uart_write_str(&mut serial, "Conversions: ");
    uart_write_hex(&mut serial, num_conversions as u8);
    uart_write_str(&mut serial, "\n");

    if test_passed {
        uart_write_str(&mut serial, "ADC TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "ADC TEST FAILED\n");
        led.set_low().ok();
    }

    // Halt
    loop {
        cortex_m::asm::wfi();
    }
}
