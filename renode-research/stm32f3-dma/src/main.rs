//! STM32F3 DMA Peripheral Test in Rust
//!
//! This tests DMA functionality on the STM32F303:
//! - DMA1 Channel1 memory-to-memory transfer
//! - Verifies data integrity after transfer
//! - Tests transfer complete flag
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

// Source and destination buffers (must be in SRAM, not CCM for DMA access)
static mut SRC_BUFFER: [u8; 16] = [0xAA, 0x55, 0x12, 0x34, 0xDE, 0xAD, 0xBE, 0xEF,
                                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
static mut DST_BUFFER: [u8; 16] = [0u8; 16];

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

    uart_write_str(&mut serial, "DMA Peripheral Test\n");

    let mut pass_count = 0u8;
    let mut fail_count = 0u8;

    // Enable DMA1 clock
    unsafe {
        let rcc_ptr = &*pac::RCC::ptr();
        rcc_ptr.ahbenr.modify(|_, w| w.dma1en().enabled());
    }

    let dma1 = unsafe { &*pac::DMA1::ptr() };

    // =========================================
    // Test 1: DMA1 Channel1 Memory-to-Memory
    // =========================================
    uart_write_str(&mut serial, "\nTest 1: Memory-to-Memory Transfer\n");

    // Get buffer addresses
    let src_addr = unsafe { SRC_BUFFER.as_ptr() as u32 };
    let dst_addr = unsafe { DST_BUFFER.as_mut_ptr() as u32 };

    uart_write_str(&mut serial, "SRC: 0x");
    uart_write_hex(&mut serial, ((src_addr >> 24) & 0xFF) as u8);
    uart_write_hex(&mut serial, ((src_addr >> 16) & 0xFF) as u8);
    uart_write_hex(&mut serial, ((src_addr >> 8) & 0xFF) as u8);
    uart_write_hex(&mut serial, (src_addr & 0xFF) as u8);
    uart_write_str(&mut serial, "\nDST: 0x");
    uart_write_hex(&mut serial, ((dst_addr >> 24) & 0xFF) as u8);
    uart_write_hex(&mut serial, ((dst_addr >> 16) & 0xFF) as u8);
    uart_write_hex(&mut serial, ((dst_addr >> 8) & 0xFF) as u8);
    uart_write_hex(&mut serial, (dst_addr & 0xFF) as u8);
    uart_write_str(&mut serial, "\n");

    // Configure DMA1 Channel 1
    // First disable the channel
    dma1.ch1.cr.write(|w| w.en().disabled());

    // Clear all interrupt flags for channel 1
    dma1.ifcr.write(|w| {
        w.cgif1().clear()
         .ctcif1().clear()
         .chtif1().clear()
         .cteif1().clear()
    });

    // Set number of data to transfer
    dma1.ch1.ndtr.write(|w| unsafe { w.ndt().bits(16) });

    // Set peripheral address (source for M2M)
    dma1.ch1.par.write(|w| unsafe { w.pa().bits(src_addr) });

    // Set memory address (destination)
    dma1.ch1.mar.write(|w| unsafe { w.ma().bits(dst_addr) });

    // Configure channel:
    // - MEM2MEM: Memory to memory mode
    // - PL: Priority level high
    // - MSIZE: Memory size 8-bit
    // - PSIZE: Peripheral size 8-bit
    // - MINC: Memory increment mode
    // - PINC: Peripheral increment mode
    // - DIR: Read from peripheral (source)
    dma1.ch1.cr.write(|w| {
        w.mem2mem().enabled()
         .pl().high()
         .msize().bits8()
         .psize().bits8()
         .minc().enabled()
         .pinc().enabled()
         .dir().from_peripheral()
         .en().enabled()
    });

    uart_write_str(&mut serial, "DMA transfer started\n");

    // Wait for transfer complete
    let mut timeout = 0u32;
    while dma1.isr.read().tcif1().is_not_complete() {
        timeout += 1;
        if timeout > 100_000 {
            break;
        }
    }

    // Disable channel
    dma1.ch1.cr.modify(|_, w| w.en().disabled());

    if dma1.isr.read().tcif1().is_complete() {
        uart_write_str(&mut serial, "Transfer complete flag: SET\n");

        // Verify data
        let mut data_ok = true;
        uart_write_str(&mut serial, "Verifying data...\n");

        for i in 0..16 {
            let src_byte = unsafe { SRC_BUFFER[i] };
            let dst_byte = unsafe { DST_BUFFER[i] };
            if src_byte != dst_byte {
                uart_write_str(&mut serial, "Mismatch at ");
                uart_write_hex(&mut serial, i as u8);
                uart_write_str(&mut serial, ": ");
                uart_write_hex(&mut serial, src_byte);
                uart_write_str(&mut serial, " != ");
                uart_write_hex(&mut serial, dst_byte);
                uart_write_str(&mut serial, "\n");
                data_ok = false;
            }
        }

        if data_ok {
            uart_write_str(&mut serial, "Data verified: PASS\n");
            pass_count += 1;
            led.set_high().ok();
        } else {
            uart_write_str(&mut serial, "Data mismatch: FAIL\n");
            fail_count += 1;
        }
    } else {
        uart_write_str(&mut serial, "Transfer timeout: FAIL\n");
        fail_count += 1;
    }

    // =========================================
    // Test 2: Verify NDTR decremented to 0
    // =========================================
    uart_write_str(&mut serial, "\nTest 2: NDTR Register\n");

    let ndtr_val = dma1.ch1.ndtr.read().ndt().bits();
    uart_write_str(&mut serial, "NDTR after transfer: ");
    uart_write_hex(&mut serial, ((ndtr_val >> 8) & 0xFF) as u8);
    uart_write_hex(&mut serial, (ndtr_val & 0xFF) as u8);
    uart_write_str(&mut serial, "\n");

    if ndtr_val == 0 {
        uart_write_str(&mut serial, "NDTR is zero: PASS\n");
        pass_count += 1;
    } else {
        uart_write_str(&mut serial, "NDTR not zero: FAIL\n");
        fail_count += 1;
    }

    // =========================================
    // Test 3: Second transfer with different data
    // =========================================
    uart_write_str(&mut serial, "\nTest 3: Second Transfer\n");

    // Modify source buffer
    unsafe {
        for i in 0..16 {
            SRC_BUFFER[i] = (i as u8) * 0x11;
        }
        for i in 0..16 {
            DST_BUFFER[i] = 0xFF; // Clear destination
        }
    }

    // Clear flags
    dma1.ifcr.write(|w| w.cgif1().clear());

    // Reconfigure and start
    dma1.ch1.ndtr.write(|w| unsafe { w.ndt().bits(16) });
    dma1.ch1.par.write(|w| unsafe { w.pa().bits(src_addr) });
    dma1.ch1.mar.write(|w| unsafe { w.ma().bits(dst_addr) });
    dma1.ch1.cr.modify(|_, w| w.en().enabled());

    // Wait for complete
    timeout = 0;
    while dma1.isr.read().tcif1().is_not_complete() {
        timeout += 1;
        if timeout > 100_000 {
            break;
        }
    }
    dma1.ch1.cr.modify(|_, w| w.en().disabled());

    // Verify
    let mut ok = true;
    for i in 0..16 {
        if unsafe { DST_BUFFER[i] } != (i as u8) * 0x11 {
            ok = false;
            break;
        }
    }

    if ok && dma1.isr.read().tcif1().is_complete() {
        uart_write_str(&mut serial, "Second transfer: PASS\n");
        pass_count += 1;
    } else {
        uart_write_str(&mut serial, "Second transfer: FAIL\n");
        fail_count += 1;
    }

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
        uart_write_str(&mut serial, "DMA TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "DMA TEST FAILED\n");
        led.set_low().ok();
    }

    loop {
        cortex_m::asm::wfi();
    }
}
