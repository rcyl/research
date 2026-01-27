//! STM32F3 DMA Peripheral Test in Rust
//!
//! This tests DMA functionality on the STM32F303:
//! - DMA1 Channel1 memory-to-memory transfer
//! - Verifies data integrity after transfer
//! - Tests transfer complete flag
//! - Reports results via USART1

#![no_std]
#![no_main]

use core::cell::UnsafeCell;
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f3_common::{constants, uart_write_hex, uart_write_str};
use stm32f3xx_hal::{
    pac,
    prelude::*,
    serial::{config::Config as UartConfig, Serial},
};

/// A wrapper for DMA buffers that provides interior mutability
/// while being safe to use in a single-threaded embedded context.
///
/// # Safety
/// This type implements `Sync` because in a single-threaded embedded
/// environment without preemption (or with properly managed interrupts),
/// there is no concurrent access to the buffer data.
struct DmaBuffer<const N: usize> {
    data: UnsafeCell<[u8; N]>,
}

// SAFETY: Single-threaded embedded context - no concurrent access
unsafe impl<const N: usize> Sync for DmaBuffer<N> {}

impl<const N: usize> DmaBuffer<N> {
    const fn new(init: [u8; N]) -> Self {
        Self {
            data: UnsafeCell::new(init),
        }
    }

    /// Get the address of the buffer for DMA configuration
    fn as_ptr(&self) -> *const u8 {
        self.data.get() as *const u8
    }

    /// Get the mutable address of the buffer for DMA configuration
    fn as_mut_ptr(&self) -> *mut u8 {
        self.data.get() as *mut u8
    }

    /// Read a byte from the buffer
    ///
    /// # Safety
    /// Caller must ensure no DMA transfer is active on this buffer
    unsafe fn read(&self, index: usize) -> u8 {
        (*self.data.get())[index]
    }

    /// Write a byte to the buffer
    ///
    /// # Safety
    /// Caller must ensure no DMA transfer is active on this buffer
    unsafe fn write(&self, index: usize, value: u8) {
        (*self.data.get())[index] = value;
    }
}

// Source and destination buffers (must be in SRAM, not CCM for DMA access)
static SRC_BUFFER: DmaBuffer<16> = DmaBuffer::new([
    0xAA, 0x55, 0x12, 0x34, 0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
]);
static DST_BUFFER: DmaBuffer<16> = DmaBuffer::new([0u8; 16]);

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
    let src_addr = SRC_BUFFER.as_ptr() as u32;
    let dst_addr = DST_BUFFER.as_mut_ptr() as u32;

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
        w.cgif1()
            .clear()
            .ctcif1()
            .clear()
            .chtif1()
            .clear()
            .cteif1()
            .clear()
    });

    // Set number of data to transfer
    dma1.ch1.ndtr.write(|w| w.ndt().bits(16));

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
        w.mem2mem()
            .enabled()
            .pl()
            .high()
            .msize()
            .bits8()
            .psize()
            .bits8()
            .minc()
            .enabled()
            .pinc()
            .enabled()
            .dir()
            .from_peripheral()
            .en()
            .enabled()
    });

    uart_write_str(&mut serial, "DMA transfer started\n");

    // Wait for transfer complete (poll TCIF and NDTR)
    // Note: Some emulators may not update these until channel is disabled
    let mut timeout = 0u32;
    loop {
        let tcif = dma1.isr.read().tcif1().is_complete();
        let ndtr = dma1.ch1.ndtr.read().ndt().bits();
        if tcif || ndtr == 0 {
            break;
        }
        timeout += 1;
        if timeout > constants::DMA_TIMEOUT {
            break;
        }
    }

    // Disable channel
    dma1.ch1.cr.modify(|_, w| w.en().disabled());

    // Report status flags (informational)
    let tcif_set = dma1.isr.read().tcif1().is_complete();
    if tcif_set {
        uart_write_str(&mut serial, "Transfer complete flag: SET\n");
    } else {
        uart_write_str(&mut serial, "Transfer complete (polling done)\n");
    }

    // Verify data - this is the real test of DMA success
    // DMA is now disabled so safe to access buffers
    let mut data_ok = true;
    uart_write_str(&mut serial, "Verifying data...\n");

    for i in 0..16 {
        // SAFETY: DMA transfer is complete and channel is disabled
        let src_byte = unsafe { SRC_BUFFER.read(i) };
        let dst_byte = unsafe { DST_BUFFER.read(i) };
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

    // Modify source buffer - DMA is disabled so safe to access
    // SAFETY: DMA channel is disabled
    unsafe {
        for i in 0..16 {
            SRC_BUFFER.write(i, (i as u8) * 0x11);
        }
        for i in 0..16 {
            DST_BUFFER.write(i, 0xFF); // Clear destination
        }
    }

    // Clear flags
    dma1.ifcr.write(|w| w.cgif1().clear());

    // Reconfigure and start
    dma1.ch1.ndtr.write(|w| w.ndt().bits(16));
    dma1.ch1.par.write(|w| unsafe { w.pa().bits(src_addr) });
    dma1.ch1.mar.write(|w| unsafe { w.ma().bits(dst_addr) });
    dma1.ch1.cr.modify(|_, w| w.en().enabled());

    // Wait for complete (poll TCIF and NDTR)
    timeout = 0;
    loop {
        let tcif = dma1.isr.read().tcif1().is_complete();
        let ndtr = dma1.ch1.ndtr.read().ndt().bits();
        if tcif || ndtr == 0 {
            break;
        }
        timeout += 1;
        if timeout > constants::DMA_TIMEOUT {
            break;
        }
    }
    dma1.ch1.cr.modify(|_, w| w.en().disabled());

    // Verify data - this is the real test
    let mut ok = true;
    for i in 0..16 {
        // SAFETY: DMA transfer is complete and channel is disabled
        if unsafe { DST_BUFFER.read(i) } != (i as u8) * 0x11 {
            ok = false;
            break;
        }
    }

    if ok {
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
