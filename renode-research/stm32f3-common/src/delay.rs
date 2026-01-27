//! Simple delay function using NOP instructions

/// Simple delay loop using NOP instructions
///
/// # Arguments
/// * `cycles` - Number of NOP cycles to execute
#[inline]
pub fn delay(cycles: u32) {
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}
