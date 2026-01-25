# STM32F3 UART Example - Renode Platform

This project adds STM32F3 (STM32F303) support to Renode and provides UART examples in both C and Rust.

## Overview

The STM32F3 series was not natively supported in Renode. This project creates:
1. A custom platform definition (`stm32f3.repl`)
2. A board definition for STM32F3 Discovery (`stm32f3_discovery.repl`)
3. Example firmware in C and Rust

## Platform Features

| Component | Address | Description |
|-----------|---------|-------------|
| Flash | 0x08000000 | 256KB |
| SRAM | 0x20000000 | 40KB |
| CCM | 0x10000000 | 8KB Core Coupled Memory |
| USART1 | 0x40013800 | TX=PA9, RX=PA10 |
| USART2 | 0x40004400 | TX=PA2, RX=PA3 |
| USART3 | 0x40004800 | TX=PB10, RX=PB11 |
| GPIO A-F | 0x48000000+ | AHB2 bus (different from F4!) |

### Key Differences from STM32F4

| Feature | STM32F4 | STM32F3 |
|---------|---------|---------|
| GPIO Base | 0x40020000 (AHB1) | 0x48000000 (AHB2) |
| USART Peripheral | STM32_UART | STM32F7_USART (newer) |
| CCM Size | 64KB | 8KB |
| RCC Address | 0x40023800 | 0x40021000 |

## Building

### C Version

```bash
cd test-c
make
```

### Rust Version

```bash
cargo build --release
```

## Testing in Renode

### C Version
```bash
cd test-c
/opt/renode_1.16.0-dotnet_portable/renode --console --disable-xwt test-button.resc
```

### Rust Version
```bash
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

## Binary Sizes

| Version | Text | Data | BSS | Total |
|---------|------|------|-----|-------|
| C (bare metal) | 256 | 64 | 0 | **320 bytes** |
| Rust (stm32f3xx-hal) | 1296 | 0 | 4 | **1300 bytes** |

The C version is smaller because it uses raw register access without any HAL.
The Rust version includes the HAL but is still very compact.

## Files

```
stm32f3-uart/
├── stm32f3.repl          # STM32F303 CPU platform definition
├── stm32f3_discovery.repl # Board definition with button/LED
├── renode-config.resc    # Renode script for Rust
├── test-button.resc      # Button test script
├── Cargo.toml            # Rust project
├── memory.x              # Linker memory layout
├── src/main.rs           # Rust UART code
└── test-c/               # C test version
    ├── main.c
    ├── link.ld
    └── Makefile
```

## Platform Development Notes

### Creating STM32F3 Support

1. **Study existing platforms**: Used `stm32f4.repl` and `stm32f0.repl` as references
2. **GPIO address differs**: STM32F3 uses AHB2 (0x48000000) not AHB1
3. **USART peripheral**: STM32F3 uses newer USART (like F0/F7), not older UART
4. **RCC is simplified**: Used Python peripheral for basic clock enable tracking

### Renode Peripheral Models Used

- `CPU.CortexM` with `cortex-m4f` type
- `GPIOPort.STM32_GPIOPort` with 16 alternate functions
- `UART.STM32F7_USART` (compatible with F3 USART registers)
- `IRQControllers.NVIC` and `STM32F4_EXTI`
- `Miscellaneous.Button` for user button
- `Python.PythonPeripheral` for RCC simulation

## References

- [STM32F303 Datasheet](https://www.st.com/resource/en/datasheet/stm32f303cb.pdf)
- [STM32F303 Reference Manual (RM0316)](https://www.st.com/resource/en/reference_manual/dm00043574.pdf)
- [stm32f3xx-hal Rust HAL](https://github.com/stm32-rs/stm32f3xx-hal)
- [Renode Documentation](https://renode.readthedocs.io/)
