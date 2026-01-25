# STM32F3 SPI Loopback Test

A Rust-based SPI loopback test for the STM32F303 platform, tested using Renode emulation.

## Overview

This project demonstrates:
- SPI1 peripheral configuration on STM32F303
- Hardware loopback testing (MOSI -> MISO)
- Automated testing using Renode and Robot Framework

## Hardware Configuration

| Signal | Pin  | AF  |
|--------|------|-----|
| SCK    | PA5  | AF5 |
| MISO   | PA6  | AF5 |
| MOSI   | PA7  | AF5 |
| Debug TX | PA9 | AF7 |
| Debug RX | PA10 | AF7 |
| Status LED | PE9 | GPIO |

## Test Description

The test performs:
1. Initialize SPI1 at 1MHz, Mode 0 (CPOL=0, CPHA=0)
2. Send 5 test bytes: `0xAA, 0x55, 0x12, 0x34, 0xFF`
3. Verify each byte echoes back correctly via loopback
4. Report results on USART1 at 115200 baud

## Building

```bash
# Build release binary
cargo build --release
```

Output: `target/thumbv7em-none-eabihf/release/stm32f3-spi`

## Running Tests

### Prerequisites
- Renode 1.16.0 or later
- Robot Framework

### Run Automated Tests

```bash
export PATH="/opt/renode_1.16.0-dotnet_portable:$PATH"
renode-test tests/test-spi.robot
```

### Interactive Testing

```bash
renode renode-config.resc
```

Then in Renode console:
```
start
```

## Expected Output

```
SPI1 Loopback Test
SPI1 initialized
Starting loopback test...
TX: 0xAA RX: 0xAA PASS
TX: 0x55 RX: 0x55 PASS
TX: 0x12 RX: 0x12 PASS
TX: 0x34 RX: 0x34 PASS
TX: 0xFF RX: 0xFF PASS

=== Test Summary ===
Passed: 05
Failed: 00
SPI TEST PASSED
```

## Project Structure

```
stm32f3-spi/
├── .cargo/
│   └── config.toml      # Cargo build configuration
├── src/
│   └── main.rs          # SPI test application
├── tests/
│   └── test-spi.robot   # Robot Framework tests
├── Cargo.toml           # Rust dependencies
├── build.rs             # Build script
├── memory.x             # Linker script
├── stm32f3_spi.repl     # Renode platform description
├── renode-config.resc   # Renode interactive script
└── README.md
```

## Renode Platform

The `stm32f3_spi.repl` extends the base STM32F3 platform with:
- `SPI.SPILoopback` attached to SPI1 for loopback testing
- User button on PA0
- LED on PE9

## Test Results

| Test Case | Status | Duration |
|-----------|--------|----------|
| Should Initialize SPI And Report | PASS | 0.65s |
| Should Pass SPI Loopback Test | PASS | 0.20s |
| Should Report Test Summary | PASS | 0.19s |

## Dependencies

- `cortex-m` - Cortex-M processor support
- `cortex-m-rt` - Runtime for Cortex-M
- `stm32f3xx-hal` - Hardware abstraction layer for STM32F3
- `panic-halt` - Panic handler
- `embedded-hal` - Embedded HAL traits
- `nb` - Non-blocking abstractions

## License

MIT
