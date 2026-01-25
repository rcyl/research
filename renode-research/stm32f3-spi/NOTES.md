# STM32F3 SPI Testing Notes

## Objective
Create a simple Rust SPI test for the STM32F303 platform and verify it works using Renode's test framework.

## Approach
1. Use SPI1 peripheral on STM32F303
   - SPI1 is on APB2 at 0x40013000
   - Pins: PA5 (SCK), PA6 (MISO), PA7 (MOSI), PA4 (NSS) - AF5
2. Create a loopback test where MOSI connects to MISO
3. Send test bytes and verify received data matches
4. Output results via USART1 for verification

## SPI1 Configuration
- STM32F303 SPI1 pins (AF5):
  - PA5: SPI1_SCK
  - PA6: SPI1_MISO
  - PA7: SPI1_MOSI
  - PA4: SPI1_NSS (optional, using software CS)

## Renode Testing
- Used Robot Framework for automated testing
- Created terminal tester on USART1 to verify SPI test results
- Used `SPI.SPILoopback` peripheral attached to spi1 for loopback testing
- Found SPILoopback usage example in `/opt/renode_1.16.0-dotnet_portable/platforms/cpus/litex_linux_vexriscv_sdcard.repl`

## Files Created
- Cargo.toml - Rust project configuration
- src/main.rs - SPI test code
- memory.x - Linker script for STM32F303
- build.rs - Build script
- .cargo/config.toml - Cargo configuration
- stm32f3_spi.repl - Platform description with SPI loopback
- renode-config.resc - Renode script for interactive testing
- tests/test-spi.robot - Robot Framework automated tests

## Key Learnings

### Renode SPI Loopback
- Renode provides `SPI.SPILoopback` peripheral for testing
- Attach to SPI peripheral: `spi1Loopback: SPI.SPILoopback @ spi1`
- This echoes all MOSI data back to MISO

### stm32f3xx-hal SPI API
- Use `SpiConfig::default().frequency(1.MHz())` for configuration
- `spi.transfer(&mut [byte])` returns slice with received data
- Need `nb` crate for blocking operations (though transfer is already blocking)

### Warnings Observed
- `sysbus: (tag: 'FLASH_INTERFACE')` - Flash interface not fully implemented in platform
- `spi1: Unhandled write to offset 0x4` - Some SPI register bits not implemented in STM32SPI model

## Test Results
All 3 Robot Framework tests passed:
1. Should Initialize SPI And Report - PASS (0.65s)
2. Should Pass SPI Loopback Test - PASS (0.20s)
3. Should Report Test Summary - PASS (0.19s)

Total test time: 1.18 seconds

## Progress
- [x] Created project structure
- [x] Wrote Rust SPI loopback test
- [x] Created Renode platform description
- [x] Created Robot Framework test
- [x] Fixed compilation issues (nb crate, SPI API)
- [x] Verified tests pass in Renode
