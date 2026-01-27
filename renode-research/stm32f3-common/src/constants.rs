//! Named timeout and delay constants
//!
//! These constants replace magic numbers throughout the codebase
//! with meaningful names that describe their purpose.

/// Timeout for peripheral initialization (10,000 cycles)
pub const INIT_TIMEOUT: u32 = 10_000;

/// Extended timeout for operations that may take longer (500,000 cycles)
pub const EXTENDED_TIMEOUT: u32 = 500_000;

/// Timeout for DMA transfers (100,000 cycles)
pub const DMA_TIMEOUT: u32 = 100_000;

/// Timeout for timer operations (100,000,000 cycles)
pub const TIMER_TIMEOUT: u32 = 100_000_000;

/// Timeout waiting for input events like button presses (500,000 cycles)
pub const INPUT_TIMEOUT: u32 = 500_000;

/// Short delay for peripheral stabilization (100 cycles)
pub const STABILIZATION_DELAY: u32 = 100;

/// Medium delay for general use (1,000 cycles)
pub const MEDIUM_DELAY: u32 = 1_000;

/// Long delay for operations requiring more time (10,000 cycles)
pub const LONG_DELAY: u32 = 10_000;

/// Very long delay for RTC and similar slow peripherals (100,000 cycles)
pub const VERY_LONG_DELAY: u32 = 100_000;
