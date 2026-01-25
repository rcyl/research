# Cortex-M MCU Emulation with Renode - Tutorial Summary

Source: https://interrupt.memfault.com/blog/intro-to-renode

## Overview

This tutorial demonstrates using Renode, an open-source emulator supporting Cortex-M and other ARM architectures, to develop and test firmware without physical hardware.

## Key Setup Steps

### Installation
Renode runs cross-platform (Windows, macOS, Linux) via the Mono framework. Downloads are available from the GitHub releases page.

### Creating a Machine
The fundamental workflow involves:
1. Creating a machine instance
2. Loading a platform description file
3. Loading firmware as an ELF file
4. Configuring peripherals and analyzers
5. Starting emulation

### Example Configuration
A basic `.resc` script might include:

```
$bin?=@renode-example.elf
mach create
machine LoadPlatformDescription @platforms/boards/stm32f4_discovery-kit.repl
showAnalyzer sysbus.uart2
macro reset
"""
    sysbus LoadELF $bin
"""
runMacro $reset
```

## STM32F4 Discovery Example

The tutorial provides a complete "hello world" firmware using libopencm3 that requires:

- Clock setup (GPIOD, GPIOA, USART2)
- GPIO configuration for UART transmit
- USART peripheral setup at 115200 baud
- Implementation of the `_write()` syscall for printf support

**Critical Issue**: The default STM32F4 platform description lacks CCM memory at `0x10000000`. This requires adding a custom platform file:

```
ccm: Memory.MappedMemory @ sysbus 0x10000000
    size: 0x10000
```

## Debugging Capabilities

### Function Tracing
Enable execution tracing with symbols:
```
sysbus.cpu LogFunctionNames True
logFile @/tmp/function-trace.log
```

### GDB Integration
Renode provides a built-in GDB server accessible via:
```
machine StartGdbServer 3333
```

Then connect from a separate terminal:
```
arm-none-eabi-gdb renode-example.elf
(gdb) target remote :3333
```

This enables standard GDB operations: breakpoints, stepping, inspection, and monitor commands.

## Integration Testing

Robot Framework integration allows automated testing. A sample test demonstrates:
- Setting up the emulated environment
- Creating a UART terminal tester
- Simulating button presses via `sysbus.gpioPortA.UserButton Press/Release`
- Validating expected output strings

Tests run via:
```
python -u <renode-path>/tests/run_tests.py tests/test-button.robot
```

Results generate HTML reports with pass/fail status.

## Advantages Over QEMU

The article notes that "Renode focuses on embedded devices rather than higher-level OS emulation," supporting more Cortex-M targets than QEMU's limited two TI-based platforms.

## Resources

Complete example code is available on GitHub in the Interrupt repository, including Makefiles, firmware source, platform descriptions, and test scripts.
