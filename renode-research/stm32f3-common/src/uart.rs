//! UART helper functions for debug output

/// Write a string to UART, converting \n to \r\n
///
/// # Arguments
/// * `uart` - Any type implementing `core::fmt::Write`
/// * `s` - The string to write
pub fn uart_write_str<W: core::fmt::Write>(uart: &mut W, s: &str) {
    for c in s.chars() {
        if c == '\n' {
            let _ = uart.write_char('\r');
        }
        let _ = uart.write_char(c);
    }
}

/// Write a hex byte to UART (2 hex digits)
///
/// # Arguments
/// * `uart` - Any type implementing `core::fmt::Write`
/// * `byte` - The byte to write as hex
pub fn uart_write_hex<W: core::fmt::Write>(uart: &mut W, byte: u8) {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    let _ = uart.write_char(HEX_CHARS[(byte >> 4) as usize] as char);
    let _ = uart.write_char(HEX_CHARS[(byte & 0x0F) as usize] as char);
}

/// Write a 16-bit hex value to UART (4 hex digits)
///
/// # Arguments
/// * `uart` - Any type implementing `core::fmt::Write`
/// * `value` - The 16-bit value to write as hex
pub fn uart_write_hex16<W: core::fmt::Write>(uart: &mut W, value: u16) {
    uart_write_hex(uart, (value >> 8) as u8);
    uart_write_hex(uart, (value & 0xFF) as u8);
}

/// Write a 32-bit hex value to UART (8 hex digits)
///
/// # Arguments
/// * `uart` - Any type implementing `core::fmt::Write`
/// * `value` - The 32-bit value to write as hex
pub fn uart_write_hex32<W: core::fmt::Write>(uart: &mut W, value: u32) {
    uart_write_hex(uart, ((value >> 24) & 0xFF) as u8);
    uart_write_hex(uart, ((value >> 16) & 0xFF) as u8);
    uart_write_hex(uart, ((value >> 8) & 0xFF) as u8);
    uart_write_hex(uart, (value & 0xFF) as u8);
}
