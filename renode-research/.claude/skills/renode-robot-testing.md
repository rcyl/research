# Running Renode Robot Tests for Embedded Rust Projects

## Overview

Renode is an embedded systems emulator that can run ARM Cortex-M binaries. Robot Framework tests verify firmware behavior by checking UART output and other observable effects.

## Directory Structure

```
stm32f3-<peripheral>/
├── Cargo.toml
├── src/main.rs
├── memory.x
├── build.rs
├── stm32f3_<peripheral>.repl    # Renode platform description
└── tests/
    └── test-<peripheral>.robot  # Robot Framework test
```

## Running Tests

### Basic Command

```bash
cd /src/stm32f3-<peripheral>
renode-test tests/test-<peripheral>.robot
```

### From Workspace Root

```bash
cd /src
renode-test stm32f3-gpio/tests/test-gpio.robot
```

### Build Before Testing

Always rebuild before testing to ensure the binary is up to date:

```bash
cargo build --release -p stm32f3-gpio && \
cd stm32f3-gpio && renode-test tests/test-gpio.robot
```

## Robot Test File Structure

```robot
*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_gpio.repl
${ELF}                        ${CURDIR}/../../target/thumbv7em-none-eabihf/release/stm32f3-gpio

*** Test Cases ***
Should Initialize And Report
    [Documentation]           Verify initialization message appears
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     GPIO Test Started    timeout=5
```

## Key Variables

| Variable | Description |
|----------|-------------|
| `${CURDIR}` | Directory containing the .robot file |
| `${PLATFORM}` | Path to .repl platform description |
| `${ELF}` | Path to compiled binary |
| `${RENODEKEYWORDS}` | Built-in Renode keywords (auto-provided) |

## Common Test Keywords

### Machine Setup
```robot
Execute Command           mach create
Execute Command           machine LoadPlatformDescription @${PLATFORM}
Execute Command           sysbus LoadELF @${ELF}
```

### UART Testing
```robot
Create Terminal Tester    sysbus.usart1
Wait For Line On Uart     Expected text    timeout=5
```

### GPIO Testing
```robot
Execute Command           sysbus.gpioPortE.led SetState true
Execute Command           sysbus.gpioPortA.button Press
Execute Command           sysbus.gpioPortA.button Release
```

### Emulation Control
```robot
Start Emulation
Execute Command           emulation RunFor "1"    # Run for 1 second
```

## Path Considerations

### Workspace Build (Recommended)
When using a Cargo workspace, binaries are in `/src/target/`:
```robot
${ELF}    ${CURDIR}/../../target/thumbv7em-none-eabihf/release/stm32f3-gpio
```

### Standalone Build
If building without workspace, binaries are in project's `target/`:
```robot
${ELF}    ${CURDIR}/../target/thumbv7em-none-eabihf/release/stm32f3-gpio
```

## Test Output

### Success
```
Tests finished successfully :)
```

### Failure
```
Some tests failed :( See the list of failed tests below and logs for details!
Failed robot critical tests:
    1. test-gpio.Should Toggle LED
```

On failure, Renode saves:
- `snapshots/*.save` - Emulation state snapshot
- `logs/*.log` - Detailed execution log

## Debugging Failed Tests

1. Check the log file in `logs/` directory
2. Verify binary path is correct and binary is freshly built
3. Increase timeout values if test is timing-sensitive
4. Check UART output expectations match firmware exactly

## Known Renode Limitations

### DMA
- TCIF (Transfer Complete Interrupt Flag) may not be set
- NDTR may only update after channel is disabled
- Memory-to-memory transfers may not copy data
- **Workaround**: Verify NDTR decrements; don't rely on data verification in tests

### Timers
- Some timer modes may not be fully emulated
- Use longer timeouts for timer-based tests

## Example: Complete Test File

```robot
*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_example.repl
${ELF}                        ${CURDIR}/../../target/thumbv7em-none-eabihf/release/stm32f3-example

*** Test Cases ***
Should Initialize
    [Documentation]           Verify firmware starts
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}
    Create Terminal Tester    sysbus.usart1
    Start Emulation
    Wait For Line On Uart     Example Test    timeout=5

Should Complete Operation
    [Documentation]           Verify operation completes
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}
    Create Terminal Tester    sysbus.usart1
    Start Emulation
    Wait For Line On Uart     Operation complete    timeout=10
    Wait For Line On Uart     TEST PASSED           timeout=5
```

## Platform Description (.repl) Files

Platform descriptions define the hardware. Example for STM32F303:

```
using "platforms/cpus/stm32f303.repl"

// Add custom peripherals or connections here
gpioPortA:
    0 -> button@0

gpioPortE:
    9 -> led@0
```

## Tips

1. **Use partial matches**: `Wait For Line On Uart` matches substrings
2. **Escape special chars**: Use `\\n` for newlines in expected text
3. **Multiple assertions**: Each test case can have multiple `Wait For Line On Uart`
4. **Test isolation**: Each test case starts fresh due to `Reset Emulation`
5. **Parallel test runs**: Each test file runs in its own Renode instance
