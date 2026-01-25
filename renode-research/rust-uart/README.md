# STM32F4 UART Example in Rust

This is a Rust port of the C UART example from the Interrupt Memfault Renode tutorial.

## Features

- UART2 output at 115200 baud
- GPIO button input on PA0 (User Button)
- LED output on PD12

## Building

```bash
# Ensure you have the ARM target installed
rustup target add thumbv7em-none-eabihf

# Build release version
cargo build --release
```

## Testing in Renode

```bash
# Run the test script
/opt/renode_1.16.0-dotnet_portable/renode --console --disable-xwt test-button.resc
```

Expected output:
```
hello world!
[TEST] Pressing button...
[TEST] Releasing button...
button pressed
[TEST] Done!
```

## Binary Size

The release build is only 1,256 bytes (text + data + bss), compared to 4,688 bytes for the C version.

## Dependencies

- `cortex-m` - Low-level access to Cortex-M processors
- `cortex-m-rt` - Runtime/startup code
- `panic-halt` - Minimal panic handler
- `stm32f4xx-hal` - Hardware abstraction layer for STM32F4

## Code Structure

```rust
#[entry]
fn main() -> ! {
    // Initialize peripherals
    let dp = pac::Peripherals::take().unwrap();
    let clocks = dp.RCC.constrain().cfgr.freeze();

    // Configure GPIO and UART
    let gpioa = dp.GPIOA.split();
    let button = gpioa.pa0.into_floating_input();
    let mut serial = Serial::tx(dp.USART2, gpioa.pa2, ...);

    // Print hello world
    uart_write_str(&mut serial, "hello world!\n");

    // Main loop - detect button press
    loop {
        if button.is_high() { ... }
    }
}
```
