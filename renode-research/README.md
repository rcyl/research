# Renode Research: Cortex-M MCU Emulation

This research follows the Interrupt Memfault tutorial on Renode for STM32F4 emulation.

**Source Tutorial:** https://interrupt.memfault.com/blog/intro-to-renode

## Overview

Renode is an open-source emulator from Antmicro that supports Cortex-M and other ARM architectures. Unlike QEMU which focuses on higher-level OS emulation, Renode is designed specifically for embedded development with extensive peripheral support.

### Key Features

- **Multi-platform support**: Cortex-M0/M3/M4/M7, RISC-V, and more
- **GDB integration**: Built-in GDB server for debugging
- **UART emulation**: Virtual serial port with analyzer
- **GPIO simulation**: Button press/release simulation
- **Robot Framework**: Automated testing support

## Tutorial Completion Summary

### 1. Environment Setup

| Component | Version/Details |
|-----------|----------------|
| Renode | 1.16.0 (ARM64 .NET portable) |
| ARM Toolchain | gcc-arm-none-eabi |
| Target Board | STM32F4 Discovery |
| Firmware Library | libopencm3 |

### 2. Firmware Build

Built the example firmware using libopencm3:
- Clock configuration (GPIOD, GPIOA, USART2)
- UART setup at 115200 baud
- GPIO configuration for LED and button
- Custom `_write()` syscall for printf support

### 3. Renode Configuration

**Main Script (renode-config.resc):**
```
$bin?=@renode-example.elf
mach create
machine LoadPlatformDescription @platforms/boards/stm32f4_discovery-kit.repl
machine LoadPlatformDescription @add-ccm.repl
showAnalyzer sysbus.usart2
machine StartGdbServer 3333
sysbus LoadELF $bin
```

**CCM Memory Fix (add-ccm.repl):**
```
ccm: Memory.MappedMemory @ sysbus 0x10000000
    size: 0x10000
```

The STM32F4 has 64KB of Core Coupled Memory (CCM) at 0x10000000 that's not included in the default platform description.

### 4. Emulation Results

**UART Output:**
```
hello world!           # On startup
button pressed         # After button release
```

**GDB Session:**
```
(gdb) target remote :3333
(gdb) info registers
r0             0x0                 0
r1             0x1                 1
pc             0x80002a2           0x80002a2 <main+170>
...
```

### 5. Button Simulation

```
sysbus.gpioPortA.UserButton Press
sysbus.gpioPortA.UserButton Release
```

The firmware detects button release (falling edge) and prints "button pressed".

## Rust Port

The C example has been ported to Rust using the `stm32f4xx-hal` crate.

### Binary Size Comparison

| Version | Text | Data | BSS | Total |
|---------|------|------|-----|-------|
| C (libopencm3) | 4564 | 108 | 16 | **4688 bytes** |
| Rust (stm32f4xx-hal) | 1252 | 0 | 4 | **1256 bytes** |

The Rust version is **73% smaller** due to zero-cost abstractions and LTO.

### Build and Run Rust Version

```bash
cd rust-uart
cargo build --release

# Test in Renode
/opt/renode_1.16.0-dotnet_portable/renode --console --disable-xwt test-button.resc
```

### Key Code Differences

**C (libopencm3):**
```c
rcc_periph_clock_enable(RCC_USART2);
usart_set_baudrate(USART2, 115200);
usart_set_mode(USART2, USART_MODE_TX);
usart_enable(USART2);
```

**Rust (stm32f4xx-hal):**
```rust
let mut serial = Serial::tx(
    dp.USART2,
    tx_pin,
    Config::default().baudrate(115200.bps()),
    &clocks,
).unwrap();
```

## Directory Structure

```
renode-research/
├── README.md           # This file
├── NOTES.md            # Research notes
├── Dockerfile          # Reproducible environment
├── docs/
│   └── intro-to-renode.md  # Tutorial documentation
├── rust-uart/          # Rust port of UART example
│   ├── Cargo.toml
│   ├── src/main.rs
│   ├── memory.x
│   ├── renode-config.resc
│   └── test-button.resc
└── interrupt/          # Cloned C example repository
    └── example/renode/
        ├── renode-example.c    # Firmware source
        ├── renode-config.resc  # Renode script
        ├── add-ccm.repl        # CCM memory fix
        └── tests/              # Robot Framework tests
```

## Quick Start

### Build and Run

```bash
# Build firmware
cd interrupt/example/renode
make

# Run in Renode
/opt/renode_1.16.0-dotnet_portable/renode --console --disable-xwt renode-config.resc

# In Renode console
start
```

### GDB Debugging

```bash
# Connect to running Renode instance
gdb-multiarch renode-example.elf
(gdb) target remote :3333
(gdb) break main
(gdb) continue
```

### Function Tracing

```
sysbus.cpu LogFunctionNames True
logFile @/tmp/function-trace.log
```

## Advantages Over QEMU

| Feature | Renode | QEMU |
|---------|--------|------|
| Cortex-M targets | Many (STM32, Nordic, etc.) | Limited (2 TI boards) |
| Peripheral models | Extensive | Minimal |
| Testing framework | Robot Framework | None |
| Multi-machine simulation | Yes | No |

## References

- [Renode Documentation](https://renode.readthedocs.io/)
- [Interrupt Blog Tutorial](https://interrupt.memfault.com/blog/intro-to-renode)
- [Renode GitHub](https://github.com/renode/renode)
- [libopencm3](https://github.com/libopencm3/libopencm3)
- [stm32f4xx-hal](https://github.com/stm32-rs/stm32f4xx-hal) - Rust HAL for STM32F4
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/)
